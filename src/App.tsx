import React, { useEffect, useState, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { VaultRecord, VaultSettings } from './types';
import { TitleBar } from './components/TitleBar';
import { SearchBar } from './components/SearchBar';
import { RecordCard } from './components/RecordCard';
import { SettingsModal } from './components/SettingsModal';
import { CaptureMenu } from './components/CaptureMenu';
import { RegionSelector } from './components/RegionSelector';
import { ImagePreviewModal } from './components/ImagePreviewModal';
import { Inbox, Shield } from 'lucide-react';

export const App: React.FC = () => {
  const [records, setRecords] = useState<VaultRecord[]>([]);
  const [settings, setSettings] = useState<VaultSettings>({
    expiration_hours: 24,
    max_items: 250,
    max_storage_mb: 200,
    detect_sensitive: true,
    autostart: false,
    global_shortcut: 'Super+Shift+V',
    is_paused: false,
  });

  const [searchQuery, setSearchQuery] = useState('');
  const [activeFilter, setActiveFilter] = useState<'all' | 'text' | 'image' | 'favorite'>('all');

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [captureMenuOpen, setCaptureMenuOpen] = useState(false);
  const [regionSelectorOpen, setRegionSelectorOpen] = useState(false);
  const [previewImageRecord, setPreviewImageRecord] = useState<VaultRecord | null>(null);

  // Carregar dados iniciais
  const loadInitialData = async () => {
    try {
      const [recs, stgs] = await Promise.all([
        invoke<VaultRecord[]>('get_records'),
        invoke<VaultSettings>('get_settings'),
      ]);
      setRecords(recs);
      setSettings(stgs);
    } catch (err) {
      console.error('Erro ao carregar dados do ClipVault:', err);
    }
  };

  useEffect(() => {
    loadInitialData();

    // Ouvintes de eventos em tempo real emitidos pelo backend Rust
    const unlistenRecordAdded = listen<VaultRecord>('vault:record-added', (event) => {
      setRecords((prev) => {
        if (prev.some((r) => r.id === event.payload.id)) return prev;
        return [event.payload, ...prev];
      });
    });

    const unlistenRecordDeleted = listen<string>('vault:record-deleted', (event) => {
      setRecords((prev) => prev.filter((r) => r.id !== event.payload));
    });

    const unlistenFavoriteToggled = listen<{ id: string; is_favorite: boolean }>(
      'vault:favorite-toggled',
      (event) => {
        setRecords((prev) =>
          prev.map((r) =>
            r.id === event.payload.id ? { ...r, is_favorite: event.payload.is_favorite } : r
          )
        );
      }
    );

    const unlistenCleared = listen('vault:history-cleared', () => {
      setRecords([]);
    });

    const unlistenPruned = listen('vault:history-pruned', () => {
      loadInitialData();
    });

    const unlistenPause = listen<boolean>('vault:pause-toggled', (event) => {
      setSettings((prev) => ({ ...prev, is_paused: event.payload }));
    });

    const unlistenTriggerCapture = listen<string>('vault:trigger-capture', (event) => {
      if (event.payload === 'full') {
        handleCaptureFullscreen();
      } else if (event.payload === 'window') {
        handleCaptureWindow();
      } else if (event.payload === 'region') {
        setRegionSelectorOpen(true);
      }
    });

    const unlistenTriggerTogglePause = listen('vault:trigger-toggle-pause', () => {
      handleTogglePause();
    });

    const unlistenTriggerClearAll = listen('vault:trigger-clear-all', () => {
      handleClearHistory();
    });

    return () => {
      unlistenRecordAdded.then((f) => f());
      unlistenRecordDeleted.then((f) => f());
      unlistenFavoriteToggled.then((f) => f());
      unlistenCleared.then((f) => f());
      unlistenPruned.then((f) => f());
      unlistenPause.then((f) => f());
      unlistenTriggerCapture.then((f) => f());
      unlistenTriggerTogglePause.then((f) => f());
      unlistenTriggerClearAll.then((f) => f());
    };
  }, []);

  // Manipuladores de Ações
  const handleCopy = async (id: string) => {
    try {
      await invoke('copy_record', { id });
    } catch (e) {
      console.error('Falha ao copiar:', e);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await invoke('delete_record', { id });
    } catch (e) {
      console.error('Falha ao excluir:', e);
    }
  };

  const handleToggleFavorite = async (id: string) => {
    try {
      await invoke('toggle_favorite', { id });
    } catch (e) {
      console.error('Falha ao favoritar:', e);
    }
  };

  const handleTogglePause = async () => {
    try {
      const active = await invoke<boolean>('toggle_monitoring');
      setSettings((prev) => ({ ...prev, is_paused: !active }));
    } catch (e) {
      console.error('Falha ao alterar pausa:', e);
    }
  };

  const handleSaveSettings = async (newSettings: VaultSettings) => {
    try {
      await invoke('update_settings', { newSettings });
      setSettings(newSettings);
    } catch (e) {
      console.error('Falha ao salvar configurações:', e);
    }
  };

  const handleClearHistory = async () => {
    try {
      await invoke('clear_history');
      setRecords([]);
    } catch (e) {
      console.error('Falha ao limpar histórico:', e);
    }
  };

  const handleRunCleanup = async (): Promise<number> => {
    try {
      return await invoke<number>('run_cleanup');
    } catch (e) {
      console.error('Falha na limpeza:', e);
      return 0;
    }
  };

  const handleCaptureFullscreen = async () => {
    try {
      await invoke('capture_screen', { mode: 'fullscreen' });
    } catch (e) {
      console.error('Erro na captura fullscreen:', e);
    }
  };

  const handleCaptureWindow = async () => {
    try {
      await invoke('capture_screen', { mode: 'window' });
    } catch (e) {
      console.error('Erro na captura de janela:', e);
    }
  };

  const handleConfirmRegion = async (x: number, y: number, width: number, height: number) => {
    try {
      await invoke('capture_custom_region', {
        x: Math.round(x),
        y: Math.round(y),
        width: Math.round(width),
        height: Math.round(height),
      });
    } catch (e) {
      console.error('Erro na captura de região:', e);
    }
  };

  // Filtragem e pesquisa
  const filteredRecords = useMemo(() => {
    return records.filter((r) => {
      // Filtro de tipo
      if (activeFilter === 'text' && r.record_type !== 'text') return false;
      if (activeFilter === 'image' && r.record_type !== 'image') return false;
      if (activeFilter === 'favorite' && !r.is_favorite) return false;

      // Filtro de texto da pesquisa
      if (searchQuery.trim()) {
        const q = searchQuery.toLowerCase();
        if (r.record_type === 'text') {
          return r.content.toLowerCase().includes(q);
        } else {
          return r.preview.toLowerCase().includes(q);
        }
      }

      return true;
    });
  }, [records, activeFilter, searchQuery]);

  return (
    <div className="flex flex-col h-screen w-screen bg-vault-950 text-slate-100 overflow-hidden font-sans border border-vault-800/80 rounded-xl shadow-2xl">
      {/* Title Bar nativa customizada com suporte a drag */}
      <TitleBar
        isPaused={settings.is_paused}
        onTogglePause={handleTogglePause}
        onOpenSettings={() => setSettingsOpen(true)}
        onOpenCaptureMenu={() => setCaptureMenuOpen(true)}
      />

      {/* Barra de Pesquisa e Filtros */}
      <SearchBar
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        activeFilter={activeFilter}
        onFilterChange={setActiveFilter}
        totalCount={filteredRecords.length}
      />

      {/* Lista de Registros */}
      <div className="flex-1 overflow-y-auto p-3 space-y-2.5">
        {filteredRecords.length > 0 ? (
          filteredRecords.map((record) => (
            <RecordCard
              key={record.id}
              record={record}
              onCopy={handleCopy}
              onDelete={handleDelete}
              onToggleFavorite={handleToggleFavorite}
              onOpenImageModal={(r) => setPreviewImageRecord(r)}
              expirationHours={settings.expiration_hours}
            />
          ))
        ) : (
          <div className="h-full flex flex-col items-center justify-center text-center p-8 space-y-3">
            <div className="p-4 rounded-2xl bg-vault-900 border border-vault-800 text-slate-500 shadow-inner">
              <Inbox className="w-8 h-8" />
            </div>
            <div className="space-y-1">
              <p className="text-sm font-semibold text-slate-300">
                {searchQuery ? 'Nenhum resultado encontrado' : 'Nenhum item no histórico'}
              </p>
              <p className="text-xs text-slate-500 max-w-xs">
                {searchQuery
                  ? 'Tente pesquisar com outros termos ou limpar os filtros.'
                  : 'Copie textos ou imagens em qualquer aplicativo para que eles apareçam aqui automaticamente.'}
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Rodapé informativo discreto */}
      <div className="h-7 px-3 bg-vault-900/40 border-t border-vault-800/50 flex items-center justify-between text-[10px] text-slate-500 flex-shrink-0">
        <span className="flex items-center space-x-1.5">
          <Shield className="w-3 h-3 text-emerald-500/80" />
          <span>ClipVault 100% Offline • Sem telemetria • Expiração: 24h</span>
        </span>
        <span className="font-mono">{settings.global_shortcut} para alternar janela</span>
      </div>

      {/* Modais */}
      <SettingsModal
        settings={settings}
        isOpen={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        onSave={handleSaveSettings}
        onClearHistory={handleClearHistory}
        onRunCleanup={handleRunCleanup}
      />

      <CaptureMenu
        isOpen={captureMenuOpen}
        onClose={() => setCaptureMenuOpen(false)}
        onCaptureFullscreen={handleCaptureFullscreen}
        onCaptureWindow={handleCaptureWindow}
        onCaptureRegion={() => setRegionSelectorOpen(true)}
      />

      <RegionSelector
        isOpen={regionSelectorOpen}
        onClose={() => setRegionSelectorOpen(false)}
        onConfirm={handleConfirmRegion}
      />

      <ImagePreviewModal
        record={previewImageRecord}
        onClose={() => setPreviewImageRecord(null)}
        onCopy={handleCopy}
      />
    </div>
  );
};

export default App;
