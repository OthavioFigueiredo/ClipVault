use crate::storage::{RecordType, StorageManager, VaultRecord};
use arboard::{Clipboard, ImageData};
use image::{ImageBuffer, Rgba};
use regex::Regex;
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use x11rb::connection::Connection;
use x11rb::protocol::xfixes::{ConnectionExt as XFixesConnectionExt, SelectionEventMask};
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, GetPropertyType, Window};

pub struct ClipboardState {
    pub is_paused: AtomicBool,
    pub last_restored_hash: Mutex<Option<u64>>,
    pub last_record_hash: Mutex<Option<u64>>,
}

impl ClipboardState {
    pub fn new(paused: bool) -> Self {
        Self {
            is_paused: AtomicBool::new(paused),
            last_restored_hash: Mutex::new(None),
            last_record_hash: Mutex::new(None),
        }
    }

    pub fn set_restoring_hash(&self, hash_val: u64) {
        let mut guard = self.last_restored_hash.lock().unwrap();
        *guard = Some(hash_val);
    }

    pub fn is_restored_hash(&self, hash_val: u64) -> bool {
        let mut guard = self.last_restored_hash.lock().unwrap();
        if let Some(val) = *guard {
            if val == hash_val {
                *guard = None; // Reset para evitar que futuras cópias manuais iguais sejam ignoradas eternamente
                return true;
            }
        }
        false
    }
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Heurísticas de proteção contra conteúdo sensível (senhas, chaves privadas, cartões de crédito)
pub fn is_sensitive_content(text: &str, window_title: Option<&str>, window_class: Option<&str>) -> bool {
    // 1. Verificar gerenciadores de senhas por nome/classe da janela
    let pw_managers = [
        "keepass", "keepassxc", "1password", "bitwarden", "lastpass",
        "dashlane", "seahorse", "authenticator", "authy", "proton pass",
        "vaultwarden", "enpass"
    ];

    if let Some(class) = window_class {
        let class_lower = class.to_lowercase();
        for pw in &pw_managers {
            if class_lower.contains(pw) {
                return true;
            }
        }
    }

    if let Some(title) = window_title {
        let title_lower = title.to_lowercase();
        for pw in &pw_managers {
            if title_lower.contains(pw) {
                return true;
            }
        }
    }

    // 2. Chaves privadas PEM (SSH / RSA / OpenSSL)
    if text.contains("-----BEGIN") && text.contains("PRIVATE KEY-----") {
        return true;
    }

    // 3. Padrões de cartões de crédito (13 a 19 dígitos com ou sem hífens/espaços)
    let cc_regex = Regex::new(r"\b(?:\d[ -]*){13,19}\b").unwrap();
    for matched in cc_regex.find_iter(text) {
        let digits: String = matched.as_str().chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 13 && digits.len() <= 19 && luhn_check(&digits) {
            return true;
        }
    }

    // 4. Tokens comuns de alta entropia (GitHub, Slack, JWT, AWS)
    if text.starts_with("ghp_") || text.starts_with("gho_") || text.starts_with("xoxb-") || text.starts_with("AKIA") {
        return true;
    }
    if text.starts_with("eyJ") && text.split('.').count() == 3 {
        return true; // JWT token
    }

    false
}

/// Algoritmo de validação de Luhn para cartões de crédito
fn luhn_check(digits: &str) -> bool {
    let mut sum = 0;
    let mut alternate = false;
    for ch in digits.chars().rev() {
        if let Some(digit) = ch.to_digit(10) {
            if alternate {
                let d = digit * 2;
                sum += if d > 9 { d - 9 } else { d };
            } else {
                sum += digit;
            }
            alternate = !alternate;
        }
    }
    sum % 10 == 0
}

/// Obtém informações da janela ativa no X11 (_NET_ACTIVE_WINDOW, WM_CLASS, _NET_WM_NAME)
fn get_active_window_info(conn: &impl Connection, root: Window) -> (Option<String>, Option<String>) {
    let net_active_window = match conn.intern_atom(false, b"_NET_ACTIVE_WINDOW") {
        Ok(cookie) => match cookie.reply() {
            Ok(reply) => reply.atom,
            Err(_) => return (None, None),
        },
        Err(_) => return (None, None),
    };

    let wm_class_atom: u32 = u32::from(AtomEnum::WM_CLASS);
    let net_wm_name: u32 = match conn.intern_atom(false, b"_NET_WM_NAME") {
        Ok(cookie) => cookie.reply().map(|r| r.atom).unwrap_or(u32::from(AtomEnum::WM_NAME)),
        Err(_) => u32::from(AtomEnum::WM_NAME),
    };

    let active_win_reply = match conn.get_property(
        false,
        root,
        net_active_window,
        AtomEnum::WINDOW,
        0,
        1,
    ) {
        Ok(cookie) => cookie.reply().ok(),
        Err(_) => None,
    };

    let active_window: Window = match active_win_reply {
        Some(prop) if prop.type_ == u32::from(AtomEnum::WINDOW) && !prop.value.is_empty() => {
            u32::from_ne_bytes(prop.value[..4].try_into().unwrap_or([0, 0, 0, 0]))
        }
        _ => return (None, None),
    };

    if active_window == 0 {
        return (None, None);
    }

    // Obter WM_CLASS
    let class_str = match conn.get_property(
        false,
        active_window,
        wm_class_atom,
        AtomEnum::STRING,
        0,
        1024,
    ) {
        Ok(cookie) => cookie.reply().ok().and_then(|prop| {
            String::from_utf8(prop.value).ok()
        }),
        Err(_) => None,
    };

    // Obter _NET_WM_NAME
    let title_str = match conn.get_property(
        false,
        active_window,
        net_wm_name,
        GetPropertyType::ANY,
        0,
        1024,
    ) {
        Ok(cookie) => cookie.reply().ok().and_then(|prop| {
            String::from_utf8(prop.value).ok()
        }),
        Err(_) => None,
    };

    (title_str, class_str)
}

/// Loop em background para monitorar eventos do Clipboard X11 via XFixes.
/// Não utiliza polling! Bloqueia no wait_for_event com 0% de CPU.
pub fn start_clipboard_monitor(
    app: AppHandle,
    storage: Arc<StorageManager>,
    state: Arc<ClipboardState>,
) {
    thread::Builder::new()
        .name("clipvault-xfixes-monitor".to_string())
        .spawn(move || {
            loop {
                // Tentar conectar ao servidor X11 local
                let (conn, screen_num) = match x11rb::connect(None) {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("[ClipVault] Falha ao conectar ao X11: {}. Nova tentativa em 3s...", e);
                        thread::sleep(Duration::from_secs(3));
                        continue;
                    }
                };

                let screen = &conn.setup().roots[screen_num];
                let root = screen.root;

                // Verificar suporte à extensão XFixes
                let query = match conn.xfixes_query_version(5, 0) {
                    Ok(cookie) => cookie.reply(),
                    Err(_) => {
                        eprintln!("[ClipVault] Extensão XFixes indisponível no servidor X11.");
                        thread::sleep(Duration::from_secs(5));
                        continue;
                    }
                };

                if query.is_err() {
                    eprintln!("[ClipVault] Erro ao inicializar XFixes. Aguardando 5s...");
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }

                // Obter o Atom CLIPBOARD (ignorando estritamente XA_PRIMARY)
                let clipboard_atom = match conn.intern_atom(false, b"CLIPBOARD") {
                    Ok(cookie) => match cookie.reply() {
                        Ok(reply) => reply.atom,
                        Err(_) => {
                            thread::sleep(Duration::from_secs(2));
                            continue;
                        }
                    },
                    Err(_) => {
                        thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                };

                // Registrar para receber eventos XFixes SelectionNotify para o clipboard convencional
                let select_res = conn.xfixes_select_selection_input(
                    root,
                    clipboard_atom,
                    SelectionEventMask::SET_SELECTION_OWNER
                        | SelectionEventMask::SELECTION_WINDOW_DESTROY
                        | SelectionEventMask::SELECTION_CLIENT_CLOSE,
                );

                if let Err(e) = select_res {
                    eprintln!("[ClipVault] Erro ao selecionar input XFixes: {}", e);
                    thread::sleep(Duration::from_secs(3));
                    continue;
                }

                if let Err(e) = conn.flush() {
                    eprintln!("[ClipVault] Falha no flush X11: {}", e);
                    thread::sleep(Duration::from_secs(3));
                    continue;
                }

                println!("[ClipVault] Monitor X11/XFixes ativo e aguardando eventos do clipboard (0% CPU)");

                // Loop de eventos X11 nativo
                loop {
                    let event = match conn.wait_for_event() {
                        Ok(ev) => ev,
                        Err(e) => {
                            eprintln!("[ClipVault] Conexão X11 interrompida: {}. Reconectando...", e);
                            break; // Quebra para reconectar
                        }
                    };

                    match event {
                        x11rb::protocol::Event::XfixesSelectionNotify(notify) => {
                            if notify.selection != clipboard_atom {
                                continue;
                            }

                            // 1. Verificar se o monitoramento está pausado
                            if state.is_paused.load(Ordering::Relaxed) || storage.is_paused() {
                                continue;
                            }

                            // Pequena pausa (50ms) para garantir que o dono da seleção processe o evento
                            thread::sleep(Duration::from_millis(50));

                            // 2. Verificar dados da janela ativa para heurística de segurança
                            let (win_title, win_class) = get_active_window_info(&conn, root);

                            // 3. Ler o conteúdo do clipboard via arboard
                            let mut cb = match Clipboard::new() {
                                Ok(c) => c,
                                Err(_) => continue,
                            };

                            // Tentar primeiro obter imagem
                            let maybe_image = cb.get_image().ok();
                            if let Some(img_data) = maybe_image {
                                let width = img_data.width as u32;
                                let height = img_data.height as u32;
                                let raw_bytes = &img_data.bytes;

                                let hash_val = calculate_hash(&raw_bytes.as_ref());

                                // Evitar loop de restauração
                                if state.is_restored_hash(hash_val) {
                                    continue;
                                }

                                // Evitar duplicata consecutiva
                                {
                                    let mut last_rec = state.last_record_hash.lock().unwrap();
                                    if *last_rec == Some(hash_val) {
                                        continue;
                                    }
                                    *last_rec = Some(hash_val);
                                }

                                // Converter Rgba para PNG
                                if let Some(img_buf) = ImageBuffer::<Rgba<u8>, _>::from_raw(
                                    width,
                                    height,
                                    raw_bytes.to_vec(),
                                ) {
                                    let mut png_bytes = Vec::new();
                                    let mut cursor = std::io::Cursor::new(&mut png_bytes);
                                    if img_buf
                                        .write_to(&mut cursor, image::ImageFormat::Png)
                                        .is_ok()
                                    {
                                        if let Ok(Some(record)) =
                                            storage.add_image_record(png_bytes, width, height)
                                        {
                                            let _ = app.emit("vault:record-added", record);
                                        }
                                    }
                                }
                                continue;
                            }

                            // Se não for imagem, tentar obter texto
                            if let Ok(text) = cb.get_text() {
                                let text_trimmed = text.trim();
                                if text_trimmed.is_empty() {
                                    continue;
                                }

                                let hash_val = calculate_hash(&text);

                                // Evitar loop se acabamos de restaurar este item
                                if state.is_restored_hash(hash_val) {
                                    continue;
                                }

                                // Evitar duplicata consecutiva
                                {
                                    let mut last_rec = state.last_record_hash.lock().unwrap();
                                    if *last_rec == Some(hash_val) {
                                        continue;
                                    }
                                    *last_rec = Some(hash_val);
                                }

                                // Checar heurística de conteúdo sensível
                                let is_sensitive = is_sensitive_content(
                                    &text,
                                    win_title.as_deref(),
                                    win_class.as_deref(),
                                );

                                // Adicionar ao histórico
                                if let Ok(Some(record)) = storage.add_text_record(text, is_sensitive) {
                                    let _ = app.emit("vault:record-added", record);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        })
        .expect("Falha ao iniciar thread monitor XFixes");
}

/// Escreve conteúdo no clipboard e previne loops
pub fn copy_to_clipboard(
    record: &VaultRecord,
    state: &ClipboardState,
) -> Result<(), String> {
    let mut cb = Clipboard::new().map_err(|e| format!("Falha ao abrir clipboard: {}", e))?;

    match record.record_type {
        RecordType::Text => {
            let hash_val = calculate_hash(&record.content);
            state.set_restoring_hash(hash_val);
            cb.set_text(&record.content)
                .map_err(|e| format!("Falha ao copiar texto: {}", e))?;
        }
        RecordType::Image => {
            let path = std::path::Path::new(&record.content);
            if !path.exists() {
                return Err("Arquivo de imagem não encontrado no cache".to_string());
            }
            let img = image::open(path)
                .map_err(|e| format!("Falha ao ler imagem para cópia: {}", e))?
                .to_rgba8();

            let (width, height) = img.dimensions();
            let raw_bytes = img.into_raw();
            let hash_val = calculate_hash(&raw_bytes);
            state.set_restoring_hash(hash_val);

            let img_data = ImageData {
                width: width as usize,
                height: height as usize,
                bytes: Cow::Owned(raw_bytes),
            };
            cb.set_image(img_data)
                .map_err(|e| format!("Falha ao copiar imagem: {}", e))?;
        }
    }

    Ok(())
}
