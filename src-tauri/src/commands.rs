use crate::generator;
use crate::wg::{get_wg_status, WgStatus};
use crate::workspace::{
    detect_network, list_servers, load_settings, save_settings, set_workspace, workspace_path,
    NetworkInfo, ServerInfo, WorkspaceError,
};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Manager};

fn wgctl_path(app: &AppHandle) -> Result<PathBuf, String> {
    if cfg!(debug_assertions) {
        let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../scripts/wgctl.sh");
        if let Ok(p) = dev.canonicalize() {
            return Ok(p);
        }
    }
    app.path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("scripts")
        .join("wgctl.sh")
        .canonicalize()
        .map_err(|e| format!("wgctl.sh not found: {e}"))
}

fn require_workspace() -> Result<PathBuf, String> {
    workspace_path()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No workspace configured".to_string())
}

fn current_user() -> Result<String, String> {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .map_err(|_| "Could not determine current user".to_string())
}

fn run_wgctl(app: &AppHandle, workspace: &Path, args: &[&str]) -> Result<String, String> {
    let script = wgctl_path(app)?;

    // Prefer passwordless sudo after one-time Authorize.
    let sudo_out = Command::new("sudo")
        .arg("-n")
        .arg(&script)
        .arg("--workspace")
        .arg(workspace)
        .args(args)
        .output();

    if let Ok(output) = sudo_out {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let needs_password = stderr.contains("a password is required")
            || stderr.contains("password is required")
            || stderr.contains("PasswordRequired");
        if output.status.success() {
            return Ok(stdout);
        }
        if !needs_password {
            // Real command failure (not auth) — surface it
            return Err(if stderr.is_empty() { stdout } else { stderr });
        }
    }

    // Fall back to pkexec (prompts password every time until Authorize).
    let output = Command::new("pkexec")
        .arg(&script)
        .arg("--workspace")
        .arg(workspace)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run pkexec: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        return Err(if stderr.is_empty() { stdout } else { stderr });
    }
    Ok(stdout)
}

fn check_passwordless(app: &AppHandle) -> bool {
    let Ok(script) = wgctl_path(app) else {
        return false;
    };
    // Probe: sudo -n true-ish by running status (or help)
    let mut cmd = Command::new("sudo");
    cmd.arg("-n").arg(&script);
    if let Ok(Some(ws)) = workspace_path().map(|w| w) {
        cmd.arg("--workspace").arg(ws).arg("status");
    } else {
        // Script will fail on missing workspace, but auth should succeed without password prompt
        cmd.arg("--workspace").arg("/tmp").arg("status");
    }
    match cmd.output() {
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            !stderr.contains("a password is required") && !stderr.contains("password is required")
        }
        Err(_) => false,
    }
}

#[tauri::command]
pub fn get_workspace() -> Result<Option<String>, String> {
    Ok(workspace_path()
        .map_err(|e| e.to_string())?
        .map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn set_workspace_cmd(path: String) -> Result<Vec<ServerInfo>, String> {
    let ws = set_workspace(&path).map_err(|e| e.to_string())?;
    list_servers(&ws).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_servers() -> Result<Vec<ServerInfo>, String> {
    let ws = require_workspace()?;
    list_servers(&ws).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_status() -> Result<WgStatus, String> {
    Ok(get_wg_status())
}

#[tauri::command]
pub fn get_network_info() -> Result<NetworkInfo, String> {
    detect_network().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_selected_server() -> Result<Option<u8>, String> {
    Ok(load_settings()
        .map_err(|e| e.to_string())?
        .selected_server)
}

#[tauri::command]
pub fn set_selected_server(index: u8) -> Result<(), String> {
    let mut settings = load_settings().map_err(|e| e.to_string())?;
    settings.selected_server = Some(index);
    save_settings(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn elevation_status(app: AppHandle) -> Result<bool, String> {
    Ok(check_passwordless(&app))
}

/// One-time authorize: install sudoers + polkit so connect/disconnect skip password prompts.
/// Safer than storing your sudo password in the app.
#[tauri::command]
pub fn install_elevation(app: AppHandle) -> Result<(), String> {
    let script = wgctl_path(&app)?;
    let script_str = script
        .to_str()
        .ok_or_else(|| "Invalid wgctl path".to_string())?
        .to_string();
    if script_str.contains(' ') {
        return Err(
            "wgctl.sh path contains spaces; move Aether to a path without spaces".into(),
        );
    }
    let user = current_user()?;

    let path = std::env::temp_dir().join(format!("aether-elevate-{}.sh", std::process::id()));
    {
        let mut file = fs::File::create(&path).map_err(|e| e.to_string())?;
        let mut perms = file.metadata().map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&path, perms).map_err(|e| e.to_string())?;

        write!(
            file,
            r##"#!/bin/bash
set -euo pipefail
SCRIPT={script}
USER_NAME={user}
RULE_FILE=/etc/sudoers.d/aether-wgctl
POLKIT_FILE=/etc/polkit-1/rules.d/50-aether.rules

mkdir -p /etc/sudoers.d
printf '%s ALL=(root) NOPASSWD: %s\n' "$USER_NAME" "$SCRIPT" > "$RULE_FILE"
chmod 440 "$RULE_FILE"
visudo -cf "$RULE_FILE"

mkdir -p /etc/polkit-1/rules.d
cat > "$POLKIT_FILE" <<'EOF'
// Aether — allow pkexec of wgctl.sh without password for sudo group
polkit.addRule(function(action, subject) {{
    if (action.id == "org.freedesktop.policykit.exec" &&
        action.lookup("program") &&
        action.lookup("program").indexOf("wgctl.sh") >= 0 &&
        subject.isInGroup("sudo")) {{
        return polkit.Result.YES;
    }}
}});
EOF
chmod 644 "$POLKIT_FILE"
echo OK
"##,
            script = shell_escape(&script_str),
            user = shell_escape(&user),
        )
        .map_err(|e| e.to_string())?;
    }

    let output = Command::new("pkexec")
        .arg("bash")
        .arg(&path)
        .output()
        .map_err(|e| format!("Failed to run installer: {e}"))?;

    let _ = fs::remove_file(&path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(if stderr.is_empty() {
            stdout.to_string()
        } else {
            stderr.to_string()
        });
    }
    Ok(())
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

#[tauri::command]
pub async fn connect(app: AppHandle, server: u8) -> Result<String, String> {
    let ws = require_workspace()?;
    run_wgctl(&app, &ws, &["up", &server.to_string()])
}

#[tauri::command]
pub async fn disconnect(app: AppHandle) -> Result<String, String> {
    let ws = require_workspace()?;
    run_wgctl(&app, &ws, &["down"])
}

#[tauri::command]
pub async fn restart(app: AppHandle, server: u8) -> Result<String, String> {
    let ws = require_workspace()?;
    run_wgctl(&app, &ws, &["restart", &server.to_string()])
}

#[tauri::command]
pub async fn regenerate_configs() -> Result<(), String> {
    let ws = require_workspace()?;
    generator::regenerate_configs(&ws)
        .await
        .map_err(|e: WorkspaceError| e.to_string())
}

#[tauri::command]
pub async fn import_config(path: String) -> Result<Vec<ServerInfo>, String> {
    let ws = require_workspace()?;
    let src = PathBuf::from(&path);
    if !src.is_file() {
        return Err("Selected path is not a file".into());
    }
    let name = src
        .file_name()
        .ok_or_else(|| "Invalid file name".to_string())?
        .to_string_lossy()
        .to_string();
    let dest = ws.join(name);
    fs::copy(&src, &dest).map_err(|e| e.to_string())?;
    generator::regenerate_configs(&ws)
        .await
        .map_err(|e: WorkspaceError| e.to_string())?;
    list_servers(&ws).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_workspace_folder() -> Result<(), String> {
    let ws = require_workspace()?;
    Command::new("xdg-open")
        .arg(ws)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
