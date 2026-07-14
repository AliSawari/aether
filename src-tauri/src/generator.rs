use crate::workspace::{
    detect_network, list_source_conf_files, parse_endpoint, parse_wg_conf, WorkspaceError,
};
use std::fs;
use std::path::Path;
use tokio::net::lookup_host;

#[derive(Debug)]
struct ParsedConfig {
    private_key: String,
    address: String,
    dns: String,
    mtu: String,
    public_key: String,
    allowed_ips: String,
    persistent_keepalive: String,
    host: String,
    port: String,
}

fn load_source_configs(workspace: &Path) -> Result<Vec<ParsedConfig>, WorkspaceError> {
    let files = list_source_conf_files(workspace)?;
    if files.is_empty() {
        return Err(WorkspaceError::Message(format!(
            "No .conf files found in {}",
            workspace.display()
        )));
    }

    let mut configs = Vec::new();
    for path in files {
        let content = fs::read_to_string(&path)?;
        let parsed = parse_wg_conf(&content);
        let endpoint = parsed.get("Endpoint").ok_or_else(|| {
            WorkspaceError::Message(format!("Missing Endpoint in {}", path.display()))
        })?;
        let (host, port) = parse_endpoint(endpoint)?;
        configs.push(ParsedConfig {
            private_key: parsed.get("PrivateKey").cloned().unwrap_or_default(),
            address: parsed.get("Address").cloned().unwrap_or_default(),
            dns: parsed
                .get("DNS")
                .cloned()
                .unwrap_or_else(|| "8.8.8.8, 8.8.4.4".into()),
            mtu: parsed.get("MTU").cloned().unwrap_or_else(|| "1300".into()),
            public_key: parsed.get("PublicKey").cloned().unwrap_or_default(),
            allowed_ips: parsed.get("AllowedIPs").cloned().unwrap_or_default(),
            persistent_keepalive: parsed
                .get("PersistentKeepalive")
                .cloned()
                .unwrap_or_else(|| "11".into()),
            host,
            port,
        });
    }
    Ok(configs)
}

fn build_config(ips: &[String], cfg: &ParsedConfig, iface: &str, gateway: &str) -> String {
    let mut post_up = String::new();
    let mut post_down = String::new();
    for ip in ips {
        post_up.push_str(&format!(
            "PostUp = ip route add {ip} via {gateway} dev {iface}\n"
        ));
        post_down.push_str(&format!(
            "PostDown = ip route del {ip} via {gateway} dev {iface}\n"
        ));
    }

    format!(
        "[Interface]\n\
         PrivateKey = {pk}\n\
         Address = {addr}\n\
         DNS = {dns}\n\
         MTU = {mtu}\n\
         \n\
         # Linux Customization \n\
         {post_up}\
         {post_down}\n\
         [Peer]\n\
         PublicKey = {pubk}\n\
         Endpoint = {host}:{port}\n\
         AllowedIPs = {allowed}\n\
         PersistentKeepalive = {keepalive}\n",
        pk = cfg.private_key,
        addr = cfg.address,
        dns = cfg.dns,
        mtu = cfg.mtu,
        post_up = post_up,
        post_down = post_down,
        pubk = cfg.public_key,
        host = cfg.host,
        port = cfg.port,
        allowed = cfg.allowed_ips,
        keepalive = cfg.persistent_keepalive,
    )
}

async fn resolve_host(host: &str, port: &str) -> Result<Vec<String>, WorkspaceError> {
    let addr = format!("{host}:{port}");
    let addrs = lookup_host(&addr).await.map_err(|e| {
        WorkspaceError::Message(format!("DNS resolve failed for {host}: {e}"))
    })?;
    let ips: Vec<String> = addrs
        .filter_map(|a| match a.ip() {
            std::net::IpAddr::V4(v4) => Some(v4.to_string()),
            _ => None,
        })
        .collect();
    if ips.is_empty() {
        return Err(WorkspaceError::Message(format!(
            "No IPv4 addresses for {host}"
        )));
    }
    Ok(ips)
}

fn remove_old_configs(workspace: &Path) -> Result<(), WorkspaceError> {
    for entry in fs::read_dir(workspace)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if crate::workspace::is_generated_conf_name(&name) {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

pub async fn regenerate_configs(workspace: &Path) -> Result<(), WorkspaceError> {
    let configs = load_source_configs(workspace)?;
    let net = detect_network()?;

    remove_old_configs(workspace)?;

    for (i, cfg) in configs.iter().enumerate() {
        let n = i + 1;
        let ips = resolve_host(&cfg.host, &cfg.port).await?;

        let wifi = build_config(&ips, cfg, &net.interface_wifi, &net.gateway);
        let eth = build_config(&ips, cfg, &net.interface_eth, &net.gateway);

        fs::write(workspace.join(format!("smart{n}-wifi.conf")), wifi)?;
        fs::write(workspace.join(format!("smart{n}-eth.conf")), eth)?;
    }

    Ok(())
}
