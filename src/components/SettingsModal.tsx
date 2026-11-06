import React, { useState } from 'react';
import { VaultSettings } from '../types';
import { X, Shield, Clock, HardDrive, Trash2, AlertTriangle, Check, RefreshCw } from 'lucide-react';

interface SettingsModalProps {
  settings: VaultSettings;
  isOpen: boolean;
  onClose: () => void;
  onSave: (newSettings: VaultSettings) => Promise<void>;
  onClearHistory: () => Promise<void>;
  onRunCleanup: () => Promise<number>;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({
  settings,
  isOpen,
  onClose,
  onSave,
  onClearHistory,
  onRunCleanup,
}) => {
  if (!isOpen) return null;

  const [form, setForm] = useState<VaultSettings>({ ...settings });
  const [cleaning, setCleaning] = useState(false);
  const [cleanedMsg, setCleanedMsg] = useState<string | null>(null);
  const [confirmClear, setConfirmClear] = useState(false);
  const [savedMsg, setSavedMsg] = useState(false);

  const handleSave = async () => {
    await onSave(form);
    setSavedMsg(true);
    setTimeout(() => {
      setSavedMsg(false);
      onClose();
    }, 700);
  };

  const handleRunCleanup = async () => {
    setCleaning(true);
    try {
      const removed = await onRunCleanup();
      setCleanedMsg(`${removed} item(ns) expirado(s) removido(s)`);
      setTimeout(() => setCleanedMsg(null), 3000);
    } finally {
      setCleaning(false);
    }
  };

  const handleClearHistory = async () => {
    if (!confirmClear) {
      setConfirmClear(true);
      return;
    }
    await onClearHistory();
    setConfirmClear(false);
    onClose();
  };

  return (
    <div
      onClick={onClose}
      className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4 select-none"
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-full max-w-lg bg-vault-900 border border-vault-700/80 rounded-2xl shadow-2xl overflow-hidden flex flex-col"
      >
        {/* Header */}
        <div className="px-5 py-4 border-b border-vault-800 flex items-center justify-between bg-vault-950">
          <div className="flex items-center space-x-2.5">
            <Shield className="w-5 h-5 text-indigo-400" />
            <h2 className="text-sm font-semibold text-slate-100">Configurações do ClipVault</h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 text-slate-400 hover:text-slate-200 rounded-lg hover:bg-vault-800 transition"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content */}
        <div className="p-5 space-y-5 overflow-y-auto max-h-[70vh]">
          {/* 1. Expiração automática */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <label className="text-xs font-semibold text-slate-200 flex items-center space-x-1.5">
                <Clock className="w-3.5 h-3.5 text-cyan-400" />
                <span>Expiração Automática</span>
              </label>
              <span className="text-xs font-mono text-cyan-400 font-medium">
                {form.expiration_hours} horas
              </span>
            </div>
            <input
              type="range"
              min="1"
              max="72"
              value={form.expiration_hours}
              onChange={(e) => setForm({ ...form, expiration_hours: Number(e.target.value) })}
              className="w-full accent-cyan-500 bg-vault-800 h-1.5 rounded-lg appearance-none cursor-pointer"
            />
            <p className="text-[11px] text-slate-500">
              Registros e imagens não fixados são apagados permanentemente após o tempo configurado (padrão 24h).
            </p>
          </div>

          <div className="h-[1px] bg-vault-800" />

          {/* 2. Limites de Armazenamento */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-slate-200 flex items-center space-x-1.5">
                <HardDrive className="w-3.5 h-3.5 text-indigo-400" />
                <span>Máximo de Itens</span>
              </label>
              <input
                type="number"
                min="50"
                max="2000"
                value={form.max_items}
                onChange={(e) => setForm({ ...form, max_items: Number(e.target.value) })}
                className="w-full px-2.5 py-1.5 bg-vault-950 border border-vault-700 rounded-lg text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-slate-200 flex items-center space-x-1.5">
                <HardDrive className="w-3.5 h-3.5 text-indigo-400" />
                <span>Limite em Disco (MB)</span>
              </label>
              <input
                type="number"
                min="50"
                max="5000"
                value={form.max_storage_mb}
                onChange={(e) => setForm({ ...form, max_storage_mb: Number(e.target.value) })}
                className="w-full px-2.5 py-1.5 bg-vault-950 border border-vault-700 rounded-lg text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
              />
            </div>
          </div>

          <div className="h-[1px] bg-vault-800" />

          {/* 3. Toggles de Segurança e Sistema */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <span className="text-xs font-semibold text-slate-200">Proteção de Conteúdo Sensível</span>
                <p className="text-[11px] text-slate-500">
                  Detecta gerenciadores de senhas, cartões e tokens via heurística.
                </p>
              </div>
              <input
                type="checkbox"
                checked={form.detect_sensitive}
                onChange={(e) => setForm({ ...form, detect_sensitive: e.target.checked })}
                className="w-4 h-4 rounded text-indigo-600 accent-indigo-500 bg-vault-950 border-vault-700 cursor-pointer"
              />
            </div>

            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <span className="text-xs font-semibold text-slate-200">Iniciar com o Ubuntu</span>
                <p className="text-[11px] text-slate-500">
                  Adiciona entrada de inicialização automática em ~/.config/autostart/.
                </p>
              </div>
              <input
                type="checkbox"
                checked={form.autostart}
                onChange={(e) => setForm({ ...form, autostart: e.target.checked })}
                className="w-4 h-4 rounded text-indigo-600 accent-indigo-500 bg-vault-950 border-vault-700 cursor-pointer"
              />
            </div>
          </div>

          <div className="h-[1px] bg-vault-800" />

          {/* 4. Atalho Global */}
          <div className="space-y-1.5">
            <label className="text-xs font-semibold text-slate-200">Atalho Global de Abertura</label>
            <div className="flex items-center space-x-2">
              <input
                type="text"
                value={form.global_shortcut}
                onChange={(e) => setForm({ ...form, global_shortcut: e.target.value })}
                placeholder="Super+Shift+V"
                className="w-full px-2.5 py-1.5 bg-vault-950 border border-vault-700 rounded-lg text-xs font-mono text-slate-200 focus:outline-none focus:border-indigo-500"
              />
            </div>
            <p className="text-[11px] text-slate-500">Exemplos: Super+Shift+V, Alt+Shift+V, Ctrl+Shift+V</p>
          </div>

          {/* 5. Ações de Limpeza */}
          <div className="p-3 bg-vault-950/70 border border-vault-800 rounded-xl space-y-2.5">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-slate-300">Manutenção de Armazenamento</span>
              <button
                onClick={handleRunCleanup}
                disabled={cleaning}
                className="flex items-center space-x-1 px-2.5 py-1 rounded bg-vault-800 hover:bg-vault-700 text-slate-300 text-xs font-medium transition"
              >
                <RefreshCw className={`w-3 h-3 ${cleaning ? 'animate-spin' : ''}`} />
                <span>Limpar Itens Expirados</span>
              </button>
            </div>
            {cleanedMsg && <p className="text-[11px] text-emerald-400 font-medium">{cleanedMsg}</p>}

            <div className="pt-2 border-t border-vault-800/80 flex items-center justify-between">
              <span className="text-xs font-medium text-red-400">Limpeza Imediata</span>
              <button
                onClick={handleClearHistory}
                className={`flex items-center space-x-1 px-2.5 py-1 rounded text-xs font-medium transition ${
                  confirmClear
                    ? 'bg-red-600 hover:bg-red-500 text-white animate-pulse'
                    : 'bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/30'
                }`}
              >
                <Trash2 className="w-3 h-3" />
                <span>{confirmClear ? 'Clique para Confirmar' : 'Limpar Todo Histórico'}</span>
              </button>
            </div>
          </div>

          {/* Disclaimer de Segurança */}
          <div className="p-3 rounded-lg bg-vault-950 border border-vault-800 text-[11px] text-slate-500 leading-relaxed flex items-start space-x-2">
            <AlertTriangle className="w-4 h-4 text-amber-500 flex-shrink-0 mt-0.5" />
            <span>
              <strong>Aviso de Segurança:</strong> O ClipVault funciona 100% offline (sem rede ou banco de dados externo). A detecção de senhas é baseada em heurísticas locais de classes de janelas e formatos de dados conhecidos, não prometendo detecção perfeita.
            </span>
          </div>
        </div>

        {/* Footer */}
        <div className="px-5 py-3 border-t border-vault-800 bg-vault-950 flex items-center justify-end space-x-2">
          <button
            onClick={onClose}
            className="px-3 py-1.5 text-xs font-medium text-slate-400 hover:text-slate-200 transition"
          >
            Cancelar
          </button>
          <button
            onClick={handleSave}
            className="flex items-center space-x-1.5 px-4 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium transition shadow"
          >
            {savedMsg ? <Check className="w-3.5 h-3.5" /> : null}
            <span>{savedMsg ? 'Salvo!' : 'Salvar Alterações'}</span>
          </button>
        </div>
      </div>
    </div>
  );
};
