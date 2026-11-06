use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_hide = MenuItem::with_id(app, "toggle_window", "Abrir / Ocultar", true, None::<&str>)?;
    let capture_full = MenuItem::with_id(app, "capture_full", "Capturar Tela Inteira", true, None::<&str>)?;
    let capture_win = MenuItem::with_id(app, "capture_win", "Capturar Janela Ativa", true, None::<&str>)?;
    let capture_reg = MenuItem::with_id(app, "capture_reg", "Capturar Região...", true, None::<&str>)?;
    let toggle_pause = MenuItem::with_id(app, "toggle_pause", "Pausar / Retomar Monitoramento", true, None::<&str>)?;
    let clear_all = MenuItem::with_id(app, "clear_all", "Limpar Histórico", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair do ClipVault", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_hide,
            &capture_full,
            &capture_win,
            &capture_reg,
            &toggle_pause,
            &clear_all,
            &quit,
        ],
    )?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                "toggle_window" => {
                    toggle_window_visibility(app);
                }
                "capture_full" => {
                    let _ = app.emit("vault:trigger-capture", "full");
                }
                "capture_win" => {
                    let _ = app.emit("vault:trigger-capture", "window");
                }
                "capture_reg" => {
                    let _ = app.emit("vault:trigger-capture", "region");
                }
                "toggle_pause" => {
                    let _ = app.emit("vault:trigger-toggle-pause", ());
                }
                "clear_all" => {
                    let _ = app.emit("vault:trigger-clear-all", ());
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                toggle_window_visibility(app);
            }
        })
        .build(app)?;

    Ok(())
}

pub fn toggle_window_visibility(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
