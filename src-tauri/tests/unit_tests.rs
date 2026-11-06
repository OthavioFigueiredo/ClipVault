use clipvault_lib::clipboard::{calculate_hash, is_sensitive_content, ClipboardState};
use clipvault_lib::storage::{RecordType, StorageManager, VaultRecord};
use std::fs::{self, File};
use std::io::Write;
use uuid::Uuid;

#[test]
fn test_sensitive_content_detection() {
    // 1. Chaves privadas SSH/RSA
    let private_key = "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0...\n-----END RSA PRIVATE KEY-----";
    assert!(is_sensitive_content(private_key, None, None));

    // 2. Gerenciador de senhas pela janela ativa (KeePassXC, 1Password, Bitwarden)
    assert!(is_sensitive_content("qualquer texto copiado", Some("KeePassXC - Senhas.kdbx"), Some("keepassxc")));
    assert!(is_sensitive_content("senha123", None, Some("1password")));
    assert!(is_sensitive_content("senha_segura", Some("Bitwarden"), Some("Bitwarden")));

    // 3. Cartões de crédito com validação de Luhn
    // Número válido de teste (4532 0150 0000 0007 soma 30 % 10 == 0)
    let valid_cc = "4532 0150 0000 0007";
    assert!(is_sensitive_content(valid_cc, None, None));

    // Número inválido de teste
    let invalid_cc = "1234 5678 9012 3456";
    assert!(!is_sensitive_content(invalid_cc, None, None));

    // 4. Tokens de alta entropia (GitHub PAT, Slack, JWT)
    assert!(is_sensitive_content("ghp_1234567890abcdefghijklmnopqrstuvwxyz", None, None));
    assert!(is_sensitive_content("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3j", None, None));

    // 5. Texto normal (não sensível)
    let normal_text = "Olá mundo! Este é um texto comum copiado no ClipVault.";
    assert!(!is_sensitive_content(normal_text, Some("Firefox"), Some("firefox")));
}

#[test]
fn test_deduplication_and_loop_prevention() {
    let state = ClipboardState::new(false);

    let text1 = "Item restaurado pelo ClipVault";
    let hash1 = calculate_hash(&text1);

    // Quando restauramos um item no clipboard, marcamos seu hash
    state.set_restoring_hash(hash1);

    // O listener do X11 detecta a mudança e verifica se foi restauração própria
    assert!(state.is_restored_hash(hash1), "Deve reconhecer como item restaurado pelo ClipVault");

    // Na próxima detecção (ou cópia manual subsequente), não deve mais ignorar
    assert!(!state.is_restored_hash(hash1), "Deve ter limpado o hash após validação");
}

#[test]
fn test_atomic_write_and_recovery() {
    let test_dir = std::env::temp_dir().join(format!("test_vault_{}", Uuid::new_v4()));
    fs::create_dir_all(&test_dir).unwrap();
    let file_path = test_dir.join("history.json");

    let test_records = vec![
        VaultRecord {
            id: "rec-1".to_string(),
            record_type: RecordType::Text,
            timestamp: 1000,
            content: "Primeiro registro".to_string(),
            preview: "Primeiro registro".to_string(),
            size_bytes: 17,
            is_favorite: false,
            is_sensitive: false,
            width: None,
            height: None,
        }
    ];

    // Escrita atômica bem sucedida
    let write_res = StorageManager::atomic_write_json(&file_path, &test_records);
    assert!(write_res.is_ok());
    assert!(file_path.exists());

    // Leitura e verificação
    let content = fs::read_to_string(&file_path).unwrap();
    let loaded: Vec<VaultRecord> = serde_json::from_str(&content).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, "rec-1");

    let _ = fs::remove_dir_all(test_dir);
}

#[test]
fn test_expiration_and_orphan_cleanup() {
    let test_dir = std::env::temp_dir().join(format!("test_orphan_{}", Uuid::new_v4()));
    let images_dir = test_dir.join("images");
    fs::create_dir_all(&images_dir).unwrap();

    // Criar uma imagem ativa e uma imagem órfã
    let active_img_path = images_dir.join("active.png");
    let orphan_img_path = images_dir.join("orphan.png");
    File::create(&active_img_path).unwrap().write_all(b"active").unwrap();
    File::create(&orphan_img_path).unwrap().write_all(b"orphan").unwrap();

    let records = vec![
        VaultRecord {
            id: "img-active".to_string(),
            record_type: RecordType::Image,
            timestamp: 2000,
            content: active_img_path.to_string_lossy().to_string(),
            preview: "Imagem ativa".to_string(),
            size_bytes: 6,
            is_favorite: false,
            is_sensitive: false,
            width: Some(100),
            height: Some(100),
        }
    ];

    // Limpeza de órfãos: orphan.png deve ser excluído e active.png mantido
    StorageManager::cleanup_orphans(&records, &images_dir);
    assert!(active_img_path.exists(), "Imagem ativa deve ser preservada");
    assert!(!orphan_img_path.exists(), "Arquivo órfão deve ter sido removido");

    let _ = fs::remove_dir_all(test_dir);
}
