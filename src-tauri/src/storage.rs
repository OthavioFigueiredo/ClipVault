use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;

const HISTORY_FILE_NAME: &str = "history.json";
const SETTINGS_FILE_NAME: &str = "settings.json";
const IMAGES_DIR_NAME: &str = "images";
const DEFAULT_EXPIRATION_HOURS: u32 = 24;
const DEFAULT_MAX_ITEMS: usize = 250;
const DEFAULT_MAX_STORAGE_MB: u64 = 200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecordType {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultRecord {
    pub id: String,
    pub record_type: RecordType,
    pub timestamp: u64,
    pub content: String, // Text body or image file path
    pub preview: String,
    pub size_bytes: usize,
    pub is_favorite: bool,
    pub is_sensitive: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSettings {
    pub expiration_hours: u32,
    pub max_items: usize,
    pub max_storage_mb: u64,
    pub detect_sensitive: bool,
    pub autostart: bool,
    pub global_shortcut: String,
    pub is_paused: bool,
}

impl Default for VaultSettings {
    fn default() -> Self {
        Self {
            expiration_hours: DEFAULT_EXPIRATION_HOURS,
            max_items: DEFAULT_MAX_ITEMS,
            max_storage_mb: DEFAULT_MAX_STORAGE_MB,
            detect_sensitive: true,
            autostart: false,
            global_shortcut: "Super+Shift+V".to_string(),
            is_paused: false,
        }
    }
}

pub struct StorageManager {
    _base_dir: PathBuf,
    images_dir: PathBuf,
    history_file: PathBuf,
    settings_file: PathBuf,
    pub records: Mutex<Vec<VaultRecord>>,
    pub settings: Mutex<VaultSettings>,
}

impl StorageManager {
    pub fn new() -> Result<Self, String> {
        let cache_dir = dirs::cache_dir().ok_or("Não foi possível localizar o diretório de cache do usuário")?;
        let base_dir = cache_dir.join("clipvault");
        let images_dir = base_dir.join(IMAGES_DIR_NAME);
        let history_file = base_dir.join(HISTORY_FILE_NAME);
        let settings_file = base_dir.join(SETTINGS_FILE_NAME);

        fs::create_dir_all(&images_dir)
            .map_err(|e| format!("Falha ao criar diretório do ClipVault: {}", e))?;

        let settings = Self::load_settings_from_disk(&settings_file);
        let mut records = Self::load_records_from_disk(&history_file);

        // Limpeza imediata de itens expirados (24h)
        Self::prune_expired(&mut records, settings.expiration_hours, &images_dir);

        // Limpeza de arquivos órfãos no diretório de imagens
        Self::cleanup_orphans(&records, &images_dir);

        Ok(Self {
            _base_dir: base_dir,
            images_dir,
            history_file,
            settings_file,
            records: Mutex::new(records),
            settings: Mutex::new(settings),
        })
    }

    pub fn get_images_dir(&self) -> PathBuf {
        self.images_dir.clone()
    }

    fn load_settings_from_disk(path: &Path) -> VaultSettings {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(settings) = serde_json::from_str::<VaultSettings>(&content) {
                    return settings;
                }
            }
        }
        let default = VaultSettings::default();
        let _ = Self::atomic_write_json(path, &default);
        default
    }

    fn load_records_from_disk(path: &Path) -> Vec<VaultRecord> {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(records) = serde_json::from_str::<Vec<VaultRecord>>(&content) {
                    return records;
                }
            }
        }
        Vec::new()
    }

    /// Escrita atômica em disco utilizando arquivo temporário e rename
    pub fn atomic_write_json<T: Serialize>(path: &Path, data: &T) -> Result<(), String> {
        let parent = path.parent().ok_or("Diretório pai inválido")?;
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;

        let temp_file_path = parent.join(format!("{}.tmp.{}", 
            path.file_name().and_then(|n| n.to_str()).unwrap_or("file"),
            Uuid::new_v4()
        ));

        let json_bytes = serde_json::to_vec_pretty(data)
            .map_err(|e| format!("Erro de serialização JSON: {}", e))?;

        let mut file = File::create(&temp_file_path)
            .map_err(|e| format!("Falha ao criar arquivo temporário: {}", e))?;

        file.write_all(&json_bytes)
            .map_err(|e| format!("Falha ao escrever dados: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Falha ao sincronizar arquivo com o disco: {}", e))?;

        drop(file);

        fs::rename(&temp_file_path, path)
            .map_err(|e| format!("Falha na substituição atômica do arquivo: {}", e))?;

        Ok(())
    }

    pub fn save_history(&self) -> Result<(), String> {
        let records = self.records.lock().unwrap();
        Self::atomic_write_json(&self.history_file, &*records)
    }

    pub fn save_settings(&self) -> Result<(), String> {
        let settings = self.settings.lock().unwrap();
        Self::atomic_write_json(&self.settings_file, &*settings)
    }

    /// Remove itens que ultrapassaram o período de expiração (padrão 24h)
    fn prune_expired(records: &mut Vec<VaultRecord>, expiration_hours: u32, _images_dir: &Path) {
        let now = Utc::now().timestamp_millis() as u64;
        let max_age_ms = (expiration_hours as u64) * 60 * 60 * 1000;

        let mut retained = Vec::new();
        for record in records.drain(..) {
            // Itens favoritos/fixados não expiram automaticamente
            let is_expired = (now.saturating_sub(record.timestamp) > max_age_ms) && !record.is_favorite;
            if is_expired {
                if record.record_type == RecordType::Image {
                    let path = PathBuf::from(&record.content);
                    if path.exists() {
                        let _ = fs::remove_file(path);
                    }
                }
            } else {
                retained.push(record);
            }
        }
        *records = retained;
    }

    /// Garante limites de armazenamento configuráveis (itens e MB)
    pub fn enforce_limits(&self) {
        let (max_items, max_storage_bytes) = {
            let s = self.settings.lock().unwrap();
            (s.max_items, s.max_storage_mb * 1024 * 1024)
        };

        let mut records = self.records.lock().unwrap();

        // 1. Limite por quantidade de itens
        while records.len() > max_items {
            // Localizar o item não favorito mais antigo
            if let Some(pos) = records.iter().rposition(|r| !r.is_favorite) {
                let removed = records.remove(pos);
                if removed.record_type == RecordType::Image {
                    let path = PathBuf::from(&removed.content);
                    if path.exists() {
                        let _ = fs::remove_file(path);
                    }
                }
            } else {
                break;
            }
        }

        // 2. Limite por tamanho total em bytes
        let mut total_bytes: u64 = records.iter().map(|r| r.size_bytes as u64).sum();
        while total_bytes > max_storage_bytes && !records.is_empty() {
            if let Some(pos) = records.iter().rposition(|r| !r.is_favorite) {
                let removed = records.remove(pos);
                total_bytes = total_bytes.saturating_sub(removed.size_bytes as u64);
                if removed.record_type == RecordType::Image {
                    let path = PathBuf::from(&removed.content);
                    if path.exists() {
                        let _ = fs::remove_file(path);
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Limpa arquivos no diretório de imagens que não constam no histórico ativo
    pub fn cleanup_orphans(records: &[VaultRecord], images_dir: &Path) {
        let active_files: HashSet<PathBuf> = records
            .iter()
            .filter(|r| r.record_type == RecordType::Image)
            .map(|r| PathBuf::from(&r.content))
            .collect();

        if let Ok(entries) = fs::read_dir(images_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && !active_files.contains(&path) {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    /// Adiciona um novo registro de texto
    pub fn add_text_record(&self, text: String, is_sensitive: bool) -> Result<Option<VaultRecord>, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let mut records = self.records.lock().unwrap();

        // Deduplicação: se for idêntico ao registro mais recente, ignorar
        if let Some(last) = records.first() {
            if last.record_type == RecordType::Text && last.content == text {
                return Ok(None);
            }
        }

        let preview = if text.chars().count() > 200 {
            format!("{}...", text.chars().take(200).collect::<String>())
        } else {
            text.clone()
        };

        let record = VaultRecord {
            id: Uuid::new_v4().to_string(),
            record_type: RecordType::Text,
            timestamp: Utc::now().timestamp_millis() as u64,
            size_bytes: text.as_bytes().len(),
            content: text,
            preview,
            is_favorite: false,
            is_sensitive,
            width: None,
            height: None,
        };

        records.insert(0, record.clone());
        drop(records);

        self.enforce_limits();
        self.save_history()?;

        Ok(Some(record))
    }

    /// Adiciona um novo registro de imagem
    pub fn add_image_record(
        &self,
        image_bytes: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<Option<VaultRecord>, String> {
        let id = Uuid::new_v4().to_string();
        let file_name = format!("{}.png", id);
        let file_path = self.images_dir.join(&file_name);

        fs::write(&file_path, &image_bytes)
            .map_err(|e| format!("Falha ao salvar imagem em cache: {}", e))?;

        let size_bytes = image_bytes.len();
        let preview = format!("{}×{} PNG ({} KB)", width, height, size_bytes / 1024);

        let record = VaultRecord {
            id,
            record_type: RecordType::Image,
            timestamp: Utc::now().timestamp_millis() as u64,
            size_bytes,
            content: file_path.to_string_lossy().to_string(),
            preview,
            is_favorite: false,
            is_sensitive: false,
            width: Some(width),
            height: Some(height),
        };

        let mut records = self.records.lock().unwrap();
        records.insert(0, record.clone());
        drop(records);

        self.enforce_limits();
        self.save_history()?;

        Ok(Some(record))
    }

    /// Limpa todo o histórico e apaga arquivos de imagem
    pub fn clear_all_history(&self) -> Result<(), String> {
        let mut records = self.records.lock().unwrap();
        records.clear();
        drop(records);

        if let Ok(entries) = fs::read_dir(&self.images_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = fs::remove_file(path);
                }
            }
        }

        self.save_history()
    }

    /// Deleta um registro específico
    pub fn delete_record(&self, id: &str) -> Result<bool, String> {
        let mut records = self.records.lock().unwrap();
        if let Some(pos) = records.iter().position(|r| r.id == id) {
            let removed = records.remove(pos);
            if removed.record_type == RecordType::Image {
                let path = PathBuf::from(&removed.content);
                if path.exists() {
                    let _ = fs::remove_file(path);
                }
            }
            drop(records);
            self.save_history()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Alterna status de favorito
    pub fn toggle_favorite(&self, id: &str) -> Result<bool, String> {
        let mut records = self.records.lock().unwrap();
        if let Some(item) = records.iter_mut().find(|r| r.id == id) {
            item.is_favorite = !item.is_favorite;
            let new_state = item.is_favorite;
            drop(records);
            self.save_history()?;
            Ok(new_state)
        } else {
            Err("Registro não encontrado".to_string())
        }
    }

    /// Executa rotina de expiração
    pub fn run_expiration_check(&self) -> Result<usize, String> {
        let expiration_hours = {
            self.settings.lock().unwrap().expiration_hours
        };
        let mut records = self.records.lock().unwrap();
        let initial_len = records.len();
        Self::prune_expired(&mut records, expiration_hours, &self.images_dir);
        let removed = initial_len - records.len();
        drop(records);

        if removed > 0 {
            self.save_history()?;
        }
        Ok(removed)
    }

    pub fn get_all_records(&self) -> Vec<VaultRecord> {
        self.records.lock().unwrap().clone()
    }

    pub fn get_settings(&self) -> VaultSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update_settings(&self, new_settings: VaultSettings) -> Result<(), String> {
        let mut s = self.settings.lock().unwrap();
        *s = new_settings;
        drop(s);
        self.save_settings()
    }

    pub fn set_paused(&self, paused: bool) -> Result<(), String> {
        let mut s = self.settings.lock().unwrap();
        s.is_paused = paused;
        drop(s);
        self.save_settings()
    }

    pub fn is_paused(&self) -> bool {
        self.settings.lock().unwrap().is_paused
    }
}
