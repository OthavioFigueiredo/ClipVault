import React from 'react';
import { VaultRecord } from '../types';
import { convertFileSrc } from '@tauri-apps/api/core';
import { X, Copy } from 'lucide-react';

interface ImagePreviewModalProps {
  record: VaultRecord | null;
  onClose: () => void;
  onCopy: (id: string) => Promise<void>;
}

export const ImagePreviewModal: React.FC<ImagePreviewModalProps> = ({
  record,
  onClose,
  onCopy,
}) => {
  if (!record || record.record_type !== 'image') return null;

  return (
    <div
      onClick={onClose}
      className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4 select-none animate-fadeIn"
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="max-w-4xl max-h-[90vh] bg-vault-900 border border-vault-700/80 rounded-2xl overflow-hidden shadow-2xl flex flex-col"
      >
        {/* Header */}
        <div className="px-4 py-3 border-b border-vault-800 flex items-center justify-between bg-vault-950">
          <div className="flex items-center space-x-2 text-xs text-slate-300">
            <span className="font-semibold">{record.width}×{record.height} px</span>
            <span>•</span>
            <span>{Math.round(record.size_bytes / 1024)} KB</span>
            <span>•</span>
            <span className="text-slate-500">{new Date(record.timestamp).toLocaleString()}</span>
          </div>

          <div className="flex items-center space-x-2">
            <button
              onClick={() => onCopy(record.id)}
              className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition"
            >
              <Copy className="w-3.5 h-3.5" />
              <span>Copiar</span>
            </button>
            <button
              onClick={onClose}
              className="p-1.5 text-slate-400 hover:text-slate-200 rounded-lg hover:bg-vault-800 transition"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Image body */}
        <div className="p-4 flex items-center justify-center overflow-auto max-h-[calc(90vh-60px)] bg-vault-950/40">
          <img
            src={convertFileSrc(record.content)}
            alt="Expanded screenshot or clipboard image"
            className="max-w-full max-h-full object-contain rounded-lg shadow-md"
          />
        </div>
      </div>
    </div>
  );
};
