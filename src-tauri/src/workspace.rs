use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub workspace: Option<String>,
    #[serde(default)]
    pub selected_server: Option<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerInfo {
    pub index: u8,
    pub file: String,
    pub host: String,
    pub port: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInfo {
    pub gateway: String,
    pub interface_wifi: String,
    pub interface_eth: String,
}

/// Generated connect configs live beside providers; never treat them as sources.
pub fn is_generated_conf_name(name: &str) -> bool {
    if !name.starts_with("smart") || !name.ends_with(".conf") {
        return false;
    }
    if let Some(rest) = name.strip_prefix("smart") {
        if let Some((num, suffix)) = rest.split_once('-') {
            return !num.is_empty()
                && num.chars().all(|c| c.is_ascii_digit())
                && (suffix == "wifi.conf" || suffix == "eth.conf");
        }
    }
    false
}

fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aether")
        .join("settings.json")
}

pub fn load_settings() -> Result<Settings, WorkspaceError> {
    let path = settings_path();
    if !path.exists() {
        return Ok(Settings::default());
    }
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}

pub fn save_settings(settings: &Settings) -> Result<(), WorkspaceError> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(settings)?)?;
    Ok(())
}

pub fn workspace_path() -> Result<Option<PathBuf>, WorkspaceError> {
    let settings = load_settings()?;
    Ok(settings.workspace.map(PathBuf::from))
}

pub fn set_workspace(path: &str) -> Result<PathBuf, WorkspaceError> {
    let p = PathBuf::from(path);
    if !p.is_dir() {
        return Err(WorkspaceError::Message("Path is not a directory".into()));
    }
    let mut settings = load_settings()?;
    settings.workspace = Some(path.to_string());
    save_settings(&settings)?;
    Ok(p)
}

pub fn parse_wg_conf(content: &str) -> HashMap<String, String> {
    let mut data = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('[') || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            data.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    data
}

pub fn parse_endpoint(endpoint: &str) -> Result<(String, String), WorkspaceError> {
    let Some(idx) = endpoint.rfind(':') else {
        return Err(WorkspaceError::Message(format!(
            "Invalid endpoint: {endpoint}"
        )));
    };
    Ok((
        endpoint[..idx].to_string(),
        endpoint[idx + 1..].to_string(),
    ))
}

/// Provider configs are `.conf` files directly in the workspace (not subfolders),
/// excluding generated `smartN-wifi.conf` / `smartN-eth.conf`.
pub fn list_source_conf_files(workspace: &Path) -> Result<Vec<PathBuf>, WorkspaceError> {
    if !workspace.is_dir() {
        return Ok(vec![]);
    }

    let mut files: Vec<_> = fs::read_dir(workspace)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().is_some_and(|ext| ext == "conf")
                && !is_generated_conf_name(&p.file_name().unwrap_or_default().to_string_lossy())
        })
        .collect();
    files.sort();
    Ok(files)
}

pub fn list_servers(workspace: &Path) -> Result<Vec<ServerInfo>, WorkspaceError> {
    let files = list_source_conf_files(workspace)?;
    let mut servers = Vec::new();
    for (i, path) in files.iter().enumerate() {
        let content = fs::read_to_string(path)?;
        let parsed = parse_wg_conf(&content);
        let endpoint = parsed.get("Endpoint").ok_or_else(|| {
            WorkspaceError::Message(format!("Missing Endpoint in {}", path.display()))
        })?;
        let (host, port) = parse_endpoint(endpoint)?;
        servers.push(ServerInfo {
            index: (i + 1) as u8,
            file: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            host,
            port,
            address: parsed.get("Address").cloned().unwrap_or_default(),
        });
    }
    Ok(servers)
}

pub fn detect_network() -> Result<NetworkInfo, WorkspaceError> {
    let route_out = std::process::Command::new("ip")
        .args(["route", "show", "default"])
        .output()?;
    let route = String::from_utf8_lossy(&route_out.stdout);
    let first = route.lines().next().unwrap_or("");

    let gateway = first
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|w| {
            if w[0] == "via" {
                Some(w[1].to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| WorkspaceError::Message("Could not detect gateway".into()))?;

    let default_dev = first
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|w| {
            if w[0] == "dev" {
                Some(w[1].to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "eth0".to_string());

    let link_out = std::process::Command::new("ip")
        .args(["-o", "link", "show"])
        .output()?;
    let link = String::from_utf8_lossy(&link_out.stdout);

    let mut eth = None;
    let mut wifi = None;
    for line in link.lines() {
        if let Some(iface) = line.split(':').nth(1).map(str::trim) {
            if iface == "lo" {
                continue;
            }
            if iface.starts_with('e') && eth.is_none() {
                eth = Some(iface.to_string());
            }
            if iface.starts_with('w') && wifi.is_none() {
                wifi = Some(iface.to_string());
            }
        }
    }

    Ok(NetworkInfo {
        gateway,
        interface_eth: eth.unwrap_or_else(|| default_dev.clone()),
        interface_wifi: wifi.unwrap_or(default_dev),
    })
}
