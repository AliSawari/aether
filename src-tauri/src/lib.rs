mod commands;
mod generator;
mod tray;
mod wg;
mod workspace;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_workspace,
            commands::set_workspace_cmd,
            commands::get_servers,
            commands::get_status,
            commands::get_network_info,
            commands::get_selected_server,
            commands::set_selected_server,
            commands::elevation_status,
            commands::install_elevation,
            commands::connect,
            commands::disconnect,
            commands::restart,
            commands::regenerate_configs,
            commands::import_config,
            commands::open_workspace_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
