import React from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Shield, Pause, Play, Minus, X, Camera, Settings } from 'lucide-react';

interface TitleBarProps {
  isPaused: boolean;
  onTogglePause: () => void;
  onOpenSettings: () => void;
  onOpenCaptureMenu: () => void;
}

export const TitleBar: React.FC<TitleBarProps> = ({
  isPaused,
  onTogglePause,
  onOpenSettings,
  onOpenCaptureMenu,
}) => {
  const handleMinimize = async () => {
    try {
      await invoke('hide_window');
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div
      data-tauri-drag-region
      className="h-12 border-b border-vault-800/80 bg-vault-900/95 backdrop-blur flex items-center justify-between px-3 select-none flex-shrink-0"
    >
      {/* Brand & status */}
      <div className="flex items-center space-x-2.5">
        <div className="w-7 h-7 rounded-lg bg-gradient-to-tr from-indigo-600 to-cyan-500 p-0.5 shadow-sm shadow-indigo-500/20 flex items-center justify-center">
          <Shield className="w-4 h-4 text-white" />
        </div>
        <div className="flex items-center space-x-2">
          <span className="font-semibold text-sm tracking-wide text-slate-100">ClipVault</span>
          <span className="text-[10px] uppercase font-bold tracking-wider px-1.5 py-0.5 rounded bg-vault-800 text-slate-400 border border-vault-700">
            X11
          </span>
        </div>

        {/* Monitoring status indicator */}
        <button
          onClick={onTogglePause}
          title={isPaused ? "Monitoramento pausado. Clique para retomar." : "Monitoramento ativo via XFixes. Clique para pausar."}
          className={`flex items-center space-x-1.5 px-2 py-0.5 rounded-full text-xs transition border ${
            isPaused
              ? 'bg-amber-500/10 text-amber-400 border-amber-500/30 hover:bg-amber-500/20'
              : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/20'
          }`}
        >
          <span className={`w-1.5 h-1.5 rounded-full ${isPaused ? 'bg-amber-400' : 'bg-emerald-400 shadow-sm shadow-emerald-400/50'}`} />
          <span className="font-medium text-[11px]">{isPaused ? 'Pausado' : 'Ativo'}</span>
          {isPaused ? <Play className="w-3 h-3 ml-0.5" /> : <Pause className="w-3 h-3 ml-0.5" />}
        </button>
      </div>

      {/* Action buttons & Window controls */}
      <div className="flex items-center space-x-1">
        <button
          onClick={onOpenCaptureMenu}
          title="Capturar Tela"
          className="p-1.5 rounded-md hover:bg-vault-800 text-slate-400 hover:text-cyan-400 transition"
        >
          <Camera className="w-4 h-4" />
        </button>

        <button
          onClick={onOpenSettings}
          title="Configurações e Limpeza"
          className="p-1.5 rounded-md hover:bg-vault-800 text-slate-400 hover:text-slate-200 transition"
        >
          <Settings className="w-4 h-4" />
        </button>

        <div className="h-4 w-[1px] bg-vault-800 mx-1" />

        <button
          onClick={handleMinimize}
          title="Ocultar para a bandeja"
          className="p-1.5 rounded-md hover:bg-vault-800 text-slate-400 hover:text-slate-200 transition"
        >
          <Minus className="w-4 h-4" />
        </button>

        <button
          onClick={handleMinimize}
          title="Fechar (mantém na bandeja do sistema)"
          className="p-1.5 rounded-md hover:bg-red-500/20 text-slate-400 hover:text-red-400 transition"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};
