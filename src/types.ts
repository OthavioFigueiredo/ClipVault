export type RecordType = 'text' | 'image';

export interface VaultRecord {
  id: string;
  record_type: RecordType;
  timestamp: number;
  content: string; // Texto puro ou caminho do arquivo da imagem
  preview: string;
  size_bytes: number;
  is_favorite: boolean;
  is_sensitive: boolean;
  width?: number;
  height?: number;
}

export interface VaultSettings {
  expiration_hours: number;
  max_items: number;
  max_storage_mb: number;
  detect_sensitive: boolean;
  autostart: boolean;
  global_shortcut: string;
  is_paused: boolean;
}
