import React from 'react';
import { Monitor, Square, Crop, X } from 'lucide-react';

interface CaptureMenuProps {
  isOpen: boolean;
  onClose: () => void;
  onCaptureFullscreen: () => Promise<void>;
  onCaptureWindow: () => Promise<void>;
  onCaptureRegion: () => void;
}

export const CaptureMenu: React.FC<CaptureMenuProps> = ({
  isOpen,
  onClose,
  onCaptureFullscreen,
  onCaptureWindow,
  onCaptureRegion,
}) => {
  if (!isOpen) return null;

  return (
    <div
      onClick={onClose}
      className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 select-none"
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-sm bg-vault-900 border border-vault-700/80 rounded-2xl shadow-2xl overflow-hidden p-4 space-y-3"
      >
        <div className="flex items-center justify-between border-b border-vault-800 pb-2">
          <span className="text-xs font-semibold text-slate-200">Capturar Tela (X11)</span>
          <button
            onClick={onClose}
            className="p-1 text-slate-400 hover:text-slate-200 rounded transition"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="space-y-2">
          <button
            onClick={async () => {
              onClose();
              await onCaptureFullscreen();
            }}
            className="w-full flex items-center space-x-3 p-2.5 rounded-xl bg-vault-950 hover:bg-vault-800 border border-vault-800 hover:border-indigo-500/50 text-left transition group"
          >
            <div className="p-2 rounded-lg bg-indigo-600/20 text-indigo-400 group-hover:bg-indigo-600 group-hover:text-white transition">
              <Monitor className="w-4 h-4" />
            </div>
            <div>
              <div className="text-xs font-semibold text-slate-200">Tela Inteira</div>
              <div className="text-[11px] text-slate-500">Captura todos os monitores conectados</div>
            </div>
          </button>

          <button
            onClick={async () => {
              onClose();
              await onCaptureWindow();
            }}
            className="w-full flex items-center space-x-3 p-2.5 rounded-xl bg-vault-950 hover:bg-vault-800 border border-vault-800 hover:border-cyan-500/50 text-left transition group"
          >
            <div className="p-2 rounded-lg bg-cyan-600/20 text-cyan-400 group-hover:bg-cyan-600 group-hover:text-white transition">
              <Square className="w-4 h-4" />
            </div>
            <div>
              <div className="text-xs font-semibold text-slate-200">Janela Ativa</div>
              <div className="text-[11px] text-slate-500">Captura a janela que estava ativa</div>
            </div>
          </button>

          <button
            onClick={() => {
              onClose();
              onCaptureRegion();
            }}
            className="w-full flex items-center space-x-3 p-2.5 rounded-xl bg-vault-950 hover:bg-vault-800 border border-vault-800 hover:border-emerald-500/50 text-left transition group"
          >
            <div className="p-2 rounded-lg bg-emerald-600/20 text-emerald-400 group-hover:bg-emerald-600 group-hover:text-white transition">
              <Crop className="w-4 h-4" />
            </div>
            <div>
              <div className="text-xs font-semibold text-slate-200">Região Selecionada</div>
              <div className="text-[11px] text-slate-500">Selecione uma área retangular na tela</div>
            </div>
          </button>
        </div>
      </div>
    </div>
  );
};
