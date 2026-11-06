import React, { useEffect, useRef } from 'react';
import { Search, X, FileText, Image as ImageIcon, Star, Layers } from 'lucide-react';

interface SearchBarProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  activeFilter: 'all' | 'text' | 'image' | 'favorite';
  onFilterChange: (filter: 'all' | 'text' | 'image' | 'favorite') => void;
  totalCount: number;
}

export const SearchBar: React.FC<SearchBarProps> = ({
  searchQuery,
  onSearchChange,
  activeFilter,
  onFilterChange,
  totalCount,
}) => {
  const inputRef = useRef<HTMLInputElement>(null);

  // Atalho global '/' para focar na pesquisa
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === '/' && document.activeElement !== inputRef.current) {
        e.preventDefault();
        inputRef.current?.focus();
      } else if (e.key === 'Escape' && document.activeElement === inputRef.current) {
        onSearchChange('');
        inputRef.current?.blur();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onSearchChange]);

  return (
    <div className="p-3 bg-vault-900/60 border-b border-vault-800/80 space-y-2.5">
      {/* Search Input */}
      <div className="relative flex items-center">
        <Search className="w-4 h-4 absolute left-3 text-slate-400 pointer-events-none" />
        <input
          ref={inputRef}
          type="text"
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder="Pesquisar histórico... (pressione / para focar)"
          className="w-full pl-9 pr-9 py-1.5 bg-vault-950 border border-vault-700/80 rounded-lg text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500/30 transition"
        />
        {searchQuery ? (
          <button
            onClick={() => onSearchChange('')}
            className="absolute right-2.5 p-1 text-slate-400 hover:text-slate-200 rounded transition"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        ) : (
          <kbd className="absolute right-2.5 px-1.5 py-0.5 text-[10px] font-mono text-slate-500 bg-vault-800 border border-vault-700 rounded">
            /
          </kbd>
        )}
      </div>

      {/* Filter Tabs & Count */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-1.5">
          <button
            onClick={() => onFilterChange('all')}
            className={`flex items-center space-x-1 px-2.5 py-1 rounded-md text-xs font-medium transition ${
              activeFilter === 'all'
                ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/40'
                : 'text-slate-400 hover:text-slate-200 hover:bg-vault-800'
            }`}
          >
            <Layers className="w-3 h-3" />
            <span>Todos</span>
          </button>

          <button
            onClick={() => onFilterChange('text')}
            className={`flex items-center space-x-1 px-2.5 py-1 rounded-md text-xs font-medium transition ${
              activeFilter === 'text'
                ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/40'
                : 'text-slate-400 hover:text-slate-200 hover:bg-vault-800'
            }`}
          >
            <FileText className="w-3 h-3" />
            <span>Textos</span>
          </button>

          <button
            onClick={() => onFilterChange('image')}
            className={`flex items-center space-x-1 px-2.5 py-1 rounded-md text-xs font-medium transition ${
              activeFilter === 'image'
                ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/40'
                : 'text-slate-400 hover:text-slate-200 hover:bg-vault-800'
            }`}
          >
            <ImageIcon className="w-3 h-3" />
            <span>Imagens</span>
          </button>

          <button
            onClick={() => onFilterChange('favorite')}
            className={`flex items-center space-x-1 px-2.5 py-1 rounded-md text-xs font-medium transition ${
              activeFilter === 'favorite'
                ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40'
                : 'text-slate-400 hover:text-slate-200 hover:bg-vault-800'
            }`}
          >
            <Star className="w-3 h-3" />
            <span>Favoritos</span>
          </button>
        </div>

        <span className="text-[11px] text-slate-500">
          {totalCount} {totalCount === 1 ? 'item' : 'itens'}
        </span>
      </div>
    </div>
  );
};
