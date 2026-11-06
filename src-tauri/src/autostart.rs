use std::fs;
use std::path::PathBuf;

const AUTOSTART_DESKTOP_ENTRY: &str = r#"[Desktop Entry]
Type=Application
Name=ClipVault
Comment=Gerenciador de Área de Transferência e Capturas de Tela
Exec=clipvault --minimized
Icon=clipvault
Terminal=false
Categories=Utility;
X-GNOME-Autostart-enabled=true
StartupNotify=false
"#;

fn get_autostart_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("autostart").join("clipvault.desktop"))
}

pub fn is_autostart_enabled() -> bool {
    if let Some(path) = get_autostart_path() {
        path.exists()
    } else {
        false
    }
}

pub fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    let path = get_autostart_path().ok_or("Diretório de autostart não encontrado")?;

    if enabled {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&path, AUTOSTART_DESKTOP_ENTRY).map_err(|e| e.to_string())?;
    } else if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }

    Ok(())
}
