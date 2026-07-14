use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let disconnect = MenuItem::with_id(app, "disconnect", "Disconnect", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let connect1 = MenuItem::with_id(app, "connect1", "Connect Server 1", true, None::<&str>)?;
    let connect2 = MenuItem::with_id(app, "connect2", "Connect Server 2", true, None::<&str>)?;
    let connect3 = MenuItem::with_id(app, "connect3", "Connect Server 3", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &connect1,
            &connect2,
            &connect3,
            &disconnect,
            &show,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::new();
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let _tray = builder
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            "disconnect" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::commands::disconnect(app).await;
                });
            }
            "connect1" => spawn_connect(app, 1),
            "connect2" => spawn_connect(app, 2),
            "connect3" => spawn_connect(app, 3),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    if win.is_visible().unwrap_or(false) {
                        let _ = win.hide();
                    } else {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn spawn_connect(app: &AppHandle, server: u8) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = crate::commands::connect(app, server).await;
    });
}
