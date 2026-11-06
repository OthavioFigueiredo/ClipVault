import React, { useState } from 'react';
import { VaultRecord } from '../types';
import { convertFileSrc } from '@tauri-apps/api/core';
import {
  Copy,
  Check,
  Star,
  Trash2,
  FileText,
  Image as ImageIcon,
  ShieldAlert,
  Eye,
  EyeOff,
  Maximize2,
  Clock,
} from 'lucide-react';

interface RecordCardProps {
  record: VaultRecord;
  onCopy: (id: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onToggleFavorite: (id: string) => Promise<void>;
  onOpenImageModal: (record: VaultRecord) => void;
  expirationHours: number;
}

export const RecordCard: React.FC<RecordCardProps> = ({
  record,
  onCopy,
  onDelete,
  onToggleFavorite,
  onOpenImageModal,
  expirationHours,
}) => {
  const [copied, setCopied] = useState(false);
  const [revealed, setRevealed] = useState(!record.is_sensitive);

  const handleCopy = async (e: React.MouseEvent) => {
    e.stopPropagation();
    await onCopy(record.id);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const handleDelete = async (e: React.MouseEvent) => {
    e.stopPropagation();
    await onDelete(record.id);
  };

  const handleToggleFavorite = async (e: React.MouseEvent) => {
    e.stopPropagation();
    await onToggleFavorite(record.id);
  };

  // Cálculo de tempo relativo
  const formatTime = (ts: number) => {
    const diff = Date.now() - ts;
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return 'agora';
    if (mins < 60) return `há ${mins}m`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `há ${hours}h`;
    const days = Math.floor(hours / 24);
    return `há ${days}d`;
  };

  // Cálculo de tempo restante para expiração
  const formatExpiration = (ts: number, hoursLimit: number) => {
    if (record.is_favorite) return 'Fixado (não expira)';
    const maxAgeMs = hoursLimit * 3600 * 1000;
    const elapsed = Date.now() - ts;
    const remainingMs = maxAgeMs - elapsed;
    if (remainingMs <= 0) return 'Expirando';
    const remainingHours = Math.floor(remainingMs / 3600000);
    if (remainingHours > 0) return `Expira em ${remainingHours}h`;
    const remainingMins = Math.floor(remainingMs / 60000);
    return `Expira em ${remainingMins}m`;
  };

  return (
    <div
      onClick={handleCopy}
      className={`group relative p-3 rounded-xl border transition cursor-pointer select-text ${
        copied
          ? 'bg-emerald-950/30 border-emerald-500/50'
          : 'bg-vault-900/80 hover:bg-vault-850 border-vault-800 hover:border-vault-700 shadow-sm'
      }`}
    >
      {/* Header Info */}
      <div className="flex items-center justify-between mb-2 select-none">
        <div className="flex items-center space-x-2">
          {record.record_type === 'text' ? (
            <div className="flex items-center space-x-1 text-slate-400">
              <FileText className="w-3.5 h-3.5 text-indigo-400" />
              <span className="text-[11px] font-medium text-slate-400">Texto</span>
              <span className="text-[10px] text-slate-500">({record.size_bytes} B)</span>
            </div>
          ) : (
            <div className="flex items-center space-x-1 text-slate-400">
              <ImageIcon className="w-3.5 h-3.5 text-cyan-400" />
              <span className="text-[11px] font-medium text-slate-400">Imagem</span>
              <span className="text-[10px] text-slate-500">
                {record.width}×{record.height} ({Math.round(record.size_bytes / 1024)} KB)
              </span>
            </div>
          )}

          {record.is_sensitive && (
            <span className="flex items-center space-x-1 px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20 text-[10px] font-medium">
              <ShieldAlert className="w-3 h-3" />
              <span>Sensível</span>
            </span>
          )}
        </div>

        {/* Timestamp & Expiration */}
        <div className="flex items-center space-x-2 text-[11px] text-slate-500">
          <span title={new Date(record.timestamp).toLocaleString()}>{formatTime(record.timestamp)}</span>
          <span className="text-slate-600">•</span>
          <span className="flex items-center space-x-1 text-[10px] text-slate-500" title={formatExpiration(record.timestamp, expirationHours)}>
            <Clock className="w-2.5 h-2.5" />
            <span>{formatExpiration(record.timestamp, expirationHours)}</span>
          </span>
        </div>
      </div>

      {/* Body Content */}
      {record.record_type === 'text' ? (
        <div className="relative">
          {record.is_sensitive && !revealed ? (
            <div className="py-2 px-3 bg-vault-950/60 rounded-lg border border-vault-800/80 flex items-center justify-between">
              <span className="text-xs text-slate-500 italic">Conteúdo sensível ocultado</span>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setRevealed(true);
                }}
                className="p-1 text-slate-400 hover:text-slate-200 rounded transition"
                title="Mostrar conteúdo"
              >
                <Eye className="w-3.5 h-3.5" />
              </button>
            </div>
          ) : (
            <div className="font-mono text-xs text-slate-200 whitespace-pre-wrap break-words max-h-32 overflow-hidden leading-relaxed">
              {record.preview}
            </div>
          )}

          {record.is_sensitive && revealed && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                setRevealed(false);
              }}
              className="absolute top-0 right-0 p-1 text-slate-500 hover:text-slate-300 rounded transition"
              title="Ocultar conteúdo sensível"
            >
              <EyeOff className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      ) : (
        <div className="relative rounded-lg overflow-hidden bg-vault-950 border border-vault-800/80 flex items-center justify-center max-h-40 group/img">
          <img
            src={convertFileSrc(record.content)}
            alt="Clipboard item"
            className="max-h-40 w-auto object-contain transition-transform duration-200 group-hover/img:scale-[1.02]"
            loading="lazy"
          />
          <button
            onClick={(e) => {
              e.stopPropagation();
              onOpenImageModal(record);
            }}
            className="absolute bottom-2 right-2 p-1.5 rounded-lg bg-vault-900/80 hover:bg-vault-800 text-slate-300 backdrop-blur border border-vault-700 opacity-0 group-hover/img:opacity-100 transition shadow"
            title="Expandir imagem"
          >
            <Maximize2 className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Action Bar (Hover & Fixed) */}
      <div className="mt-2.5 pt-2 border-t border-vault-800/60 flex items-center justify-between select-none">
        <span className="text-[11px] text-slate-500 flex items-center space-x-1">
          {copied ? (
            <span className="text-emerald-400 font-medium flex items-center space-x-1">
              <Check className="w-3 h-3" />
              <span>Copiado para o Clipboard!</span>
            </span>
          ) : (
            <span className="text-slate-500 group-hover:text-slate-400 transition">
              Clique para copiar
            </span>
          )}
        </span>

        <div className="flex items-center space-x-1">
          <button
            onClick={handleToggleFavorite}
            title={record.is_favorite ? "Desafixar" : "Fixar (não expira)"}
            className={`p-1 rounded-md transition ${
              record.is_favorite
                ? 'text-amber-400 hover:text-amber-300'
                : 'text-slate-500 hover:text-slate-300 hover:bg-vault-800'
            }`}
          >
            <Star className={`w-3.5 h-3.5 ${record.is_favorite ? 'fill-amber-400' : ''}`} />
          </button>

          <button
            onClick={handleCopy}
            title="Copiar novamente"
            className="p-1 rounded-md text-slate-500 hover:text-indigo-400 hover:bg-vault-800 transition"
          >
            {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
          </button>

          <button
            onClick={handleDelete}
            title="Excluir este item"
            className="p-1 rounded-md text-slate-500 hover:text-red-400 hover:bg-vault-800 transition"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
};
