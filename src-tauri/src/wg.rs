use regex::Regex;
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Default)]
pub struct TransferInfo {
    pub rx: String,
    pub tx: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct WgStatus {
    pub connected: bool,
    pub interface: Option<String>,
    pub server_index: Option<u8>,
    pub endpoint: Option<String>,
    pub latest_handshake: Option<String>,
    pub transfer: TransferInfo,
}

pub fn parse_wg_show(output: &str) -> WgStatus {
    let mut status = WgStatus::default();
    if output.trim().is_empty() {
        return status;
    }

    let re_iface = Regex::new(r"^interface:\s+(\S+)").unwrap();
    let re_endpoint = Regex::new(r"^\s+endpoint:\s+(.+)$").unwrap();
    let re_handshake = Regex::new(r"^\s+latest handshake:\s+(.+)$").unwrap();
    let re_transfer = Regex::new(r"^\s+transfer:\s+(.+?)\s+received,\s+(.+?)\s+sent").unwrap();
    let re_server = Regex::new(r"smart(\d+)-").unwrap();

    for line in output.lines() {
        if let Some(c) = re_iface.captures(line) {
            let iface = c[1].to_string();
            status.connected = true;
            status.interface = Some(iface.clone());
            if let Some(m) = re_server.captures(&iface) {
                status.server_index = m[1].parse().ok();
            }
        } else if let Some(c) = re_endpoint.captures(line) {
            status.endpoint = Some(c[1].trim().to_string());
        } else if let Some(c) = re_handshake.captures(line) {
            status.latest_handshake = Some(c[1].trim().to_string());
        } else if let Some(c) = re_transfer.captures(line) {
            status.transfer.rx = c[1].trim().to_string();
            status.transfer.tx = c[2].trim().to_string();
        }
    }

    status
}

pub fn get_wg_status() -> WgStatus {
    let try_cmd = |program: &str, args: &[&str]| -> String {
        Command::new(program)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default()
    };

    let output = {
        let out = try_cmd("wg", &["show"]);
        if out.trim().is_empty() {
            try_cmd("sudo", &["-n", "wg", "show"])
        } else {
            out
        }
    };

    parse_wg_show(&output)
}
