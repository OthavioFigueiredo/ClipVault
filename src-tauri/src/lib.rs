pub mod autostart;
pub mod clipboard;
pub mod screenshot;
pub mod storage;
pub mod tray;

use autostart::{is_autostart_enabled, set_autostart_enabled};
use clipboard::{copy_to_clipboard, start_clipboard_monitor, ClipboardState};
use screenshot::{capture_active_window, capture_fullscreen, capture_region};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use storage::{StorageManager, VaultRecord, VaultSettings};
use tauri::{
    command, AppHandle, Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub struct AppState {
    pub storage: Arc<StorageManager>,
    pub clipboard_state: Arc<ClipboardState>,
}

#[command]
fn get_records(state: State<'_, AppState>) -> Vec<VaultRecord> {
    state.storage.get_all_records()
}

#[command]
fn copy_record(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let records = state.storage.get_all_records();
    if let Some(record) = records.iter().find(|r| r.id == id) {
        copy_to_clipboard(record, &state.clipboard_state)
    } else {
        Err("Registro não encontrado".to_string())
    }
}

#[command]
fn delete_record(id: String, state: State<'_, AppState>, app: AppHandle) -> Result<bool, String> {
    let success = state.storage.delete_record(&id)?;
    if success {
        let _ = app.emit("vault:record-deleted", id);
    }
    Ok(success)
}

#[command]
fn toggle_favorite(id: String, state: State<'_, AppState>, app: AppHandle) -> Result<bool, String> {
    let is_fav = state.storage.toggle_favorite(&id)?;
    let _ = app.emit("vault:favorite-toggled", serde_json::json!({ "id": id, "is_favorite": is_fav }));
    Ok(is_fav)
}

#[command]
fn clear_history(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    state.storage.clear_all_history()?;
    let _ = app.emit("vault:history-cleared", ());
    Ok(())
}

#[command]
fn get_settings(state: State<'_, AppState>) -> VaultSettings {
    let mut s = state.storage.get_settings();
    s.autostart = is_autostart_enabled();
    s
}

#[command]
fn update_settings(new_settings: VaultSettings, state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let _ = set_autostart_enabled(new_settings.autostart);
    state.clipboard_state.is_paused.store(new_settings.is_paused, Ordering::Relaxed);
    state.storage.update_settings(new_settings.clone())?;
    let _ = app.emit("vault:settings-updated", new_settings);
    Ok(())
}

#[command]
fn toggle_monitoring(state: State<'_, AppState>, app: AppHandle) -> Result<bool, String> {
    let current = state.clipboard_state.is_paused.load(Ordering::Relaxed);
    let new_state = !current;
    state.clipboard_state.is_paused.store(new_state, Ordering::Relaxed);
    state.storage.set_paused(new_state)?;
    let _ = app.emit("vault:pause-toggled", new_state);
    Ok(!new_state) // Retorna true se estiver ativo
}

#[command]
fn capture_screen(mode: String, state: State<'_, AppState>, app: AppHandle) -> Result<VaultRecord, String> {
    let record = match mode.as_str() {
        "fullscreen" => capture_fullscreen(&state.storage)?,
        "window" => capture_active_window(&state.storage)?,
        _ => return Err("Modo de captura inválido".to_string()),
    };
    let _ = app.emit("vault:record-added", &record);
    Ok(record)
}

#[command]
fn capture_custom_region(
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<VaultRecord, String> {
    let record = capture_region(&state.storage, x, y, width, height)?;
    let _ = app.emit("vault:record-added", &record);
    Ok(record)
}

#[command]
fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[command]
fn run_cleanup(state: State<'_, AppState>) -> Result<usize, String> {
    state.storage.run_expiration_check()
}

pub fn run() {
    let storage_manager = Arc::new(StorageManager::new().expect("Falha ao inicializar armazenamento"));
    let initial_paused = storage_manager.is_paused();
    let clipboard_state = Arc::new(ClipboardState::new(initial_paused));

    let storage_clone_for_monitor = Arc::clone(&storage_manager);
    let clip_state_for_monitor = Arc::clone(&clipboard_state);

    let storage_clone_for_timer = Arc::clone(&storage_manager);

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            storage: Arc::clone(&storage_manager),
            clipboard_state: Arc::clone(&clipboard_state),
        })
        .setup(move |app| {
            let handle = app.handle().clone();

            // Configurar a bandeja do sistema (Tray)
            tray::setup_tray(&handle)?;

            // Iniciar o monitor X11/XFixes em segundo plano
            start_clipboard_monitor(handle.clone(), storage_clone_for_monitor, clip_state_for_monitor);

            // Registrar atalho global padrão (Super+Shift+V ou Alt+Shift+V)
            let shortcut_str = storage_manager.get_settings().global_shortcut;
            if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                let h = handle.clone();
                let _ = app.global_shortcut().on_shortcut(shortcut, move |_app, _sc, _ev| {
                    tray::toggle_window_visibility(&h);
                });
            }

            // Timer em background para expiração automática a cada 5 minutos
            let app_handle_timer = handle.clone();
            std::thread::Builder::new()
                .name("clipvault-expiration-timer".to_string())
                .spawn(move || loop {
                    std::thread::sleep(Duration::from_secs(300));
                    if let Ok(removed) = storage_clone_for_timer.run_expiration_check() {
                        if removed > 0 {
                            let _ = app_handle_timer.emit("vault:history-pruned", removed);
                        }
                    }
                })
                .expect("Falha ao iniciar timer de expiração");

            // Exibir a janela inicialmente se não for iniciado com flag --minimized
            let args: Vec<String> = std::env::args().collect();
            let start_minimized = args.iter().any(|arg| arg == "--minimized");
            if !start_minimized {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Interceptar fechamento e ocultar a janela, preservando o processo no system tray
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_records,
            copy_record,
            delete_record,
            toggle_favorite,
            clear_history,
            get_settings,
            update_settings,
            toggle_monitoring,
            capture_screen,
            capture_custom_region,
            hide_window,
            run_cleanup,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao executar aplicativo ClipVault");
}
