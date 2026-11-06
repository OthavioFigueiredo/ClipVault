import React, { useState, useRef } from 'react';
import { X } from 'lucide-react';

interface RegionSelectorProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (x: number, y: number, width: number, height: number) => Promise<void>;
}

export const RegionSelector: React.FC<RegionSelectorProps> = ({
  isOpen,
  onClose,
  onConfirm,
}) => {
  if (!isOpen) return null;

  const [startX, setStartX] = useState(0);
  const [startY, setStartY] = useState(0);
  const [currentX, setCurrentX] = useState(0);
  const [currentY, setCurrentY] = useState(0);
  const [isSelecting, setIsSelecting] = useState(false);

  const containerRef = useRef<HTMLDivElement>(null);

  const handleMouseDown = (e: React.MouseEvent) => {
    setStartX(e.clientX);
    setStartY(e.clientY);
    setCurrentX(e.clientX);
    setCurrentY(e.clientY);
    setIsSelecting(true);
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isSelecting) return;
    setCurrentX(e.clientX);
    setCurrentY(e.clientY);
  };

  const handleMouseUp = async () => {
    if (!isSelecting) return;
    setIsSelecting(false);

    const x = Math.min(startX, currentX);
    const y = Math.min(startY, currentY);
    const width = Math.abs(currentX - startX);
    const height = Math.abs(currentY - startY);

    if (width > 10 && height > 10) {
      await onConfirm(x, y, width, height);
      onClose();
    }
  };

  const rectX = Math.min(startX, currentX);
  const rectY = Math.min(startY, currentY);
  const rectW = Math.abs(currentX - startX);
  const rectH = Math.abs(currentY - startY);

  return (
    <div
      ref={containerRef}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      className="fixed inset-0 z-50 cursor-crosshair bg-black/40 backdrop-blur-[1px] select-none"
    >
      <div className="absolute top-4 left-1/2 -translate-x-1/2 px-4 py-2 rounded-full bg-vault-900/90 border border-vault-700 text-xs text-slate-200 shadow-xl flex items-center space-x-2 pointer-events-auto">
        <span>Arraste para selecionar a região da captura</span>
        <button
          onClick={(e) => {
            e.stopPropagation();
            onClose();
          }}
          className="p-1 hover:bg-vault-800 rounded text-slate-400 hover:text-slate-200 transition"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      {isSelecting && (
        <div
          style={{
            left: `${rectX}px`,
            top: `${rectY}px`,
            width: `${rectW}px`,
            height: `${rectH}px`,
          }}
          className="absolute border-2 border-cyan-400 bg-cyan-400/10 pointer-events-none rounded shadow-sm"
        >
          <div className="absolute bottom-1 right-1 px-1.5 py-0.5 rounded bg-vault-950/80 text-[10px] text-cyan-300 font-mono">
            {rectW} × {rectH}
          </div>
        </div>
      )}
    </div>
  );
};
