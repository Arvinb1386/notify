import React, { useEffect, useRef } from 'react';
import { Copy, Check, Trash2, Key, Sparkles } from 'lucide-react';

interface ContextMenuProps {
  x: number;
  y: number;
  selectedText: string;
  onCopyText: () => void;
  onCopyFullNotification?: () => void;
  onDeleteNotification?: () => void;
  onClose: () => void;
  isOtp?: boolean;
  otpCode?: string | null;
  onCopyOtp?: (code: string) => void;
}

export const GlassContextMenu: React.FC<ContextMenuProps> = ({
  x,
  y,
  selectedText,
  onCopyText,
  onCopyFullNotification,
  onDeleteNotification,
  onClose,
  isOtp,
  otpCode,
  onCopyOtp,
}) => {
  const menuRef = useRef<HTMLDivElement>(null);
  const [copied, setCopied] = React.useState(false);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [onClose]);

  // Adjust coordinates so the menu doesn't overflow window edges
  const adjustedX = Math.min(x, window.innerWidth - 220);
  const adjustedY = Math.min(y, window.innerHeight - 240);

  const handleCopy = () => {
    onCopyText();
    setCopied(true);
    setTimeout(() => {
      onClose();
    }, 400);
  };

  return (
    <div
      ref={menuRef}
      style={{ top: `${adjustedY}px`, left: `${adjustedX}px` }}
      className="fixed z-50 min-w-[200px] rounded-2xl bg-[#14161f]/80 backdrop-blur-xl border border-white/10 p-1.5 shadow-[0_20px_50px_rgba(0,0,0,0.6)] text-gray-200 text-xs flex flex-col gap-0.5 animate-in fade-in zoom-in-95 duration-150 select-none"
    >
      {/* 2FA / OTP Quick Option if available */}
      {isOtp && otpCode && onCopyOtp && (
        <>
          <button
            onClick={() => {
              onCopyOtp(otpCode);
              onClose();
            }}
            className="w-full px-3 py-2 rounded-xl text-left font-medium text-amber-300 bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/20 flex items-center justify-between transition cursor-pointer active:scale-98"
          >
            <div className="flex items-center gap-2">
              <Key size={13} className="text-amber-400" />
              <span>Copy Code: {otpCode}</span>
            </div>
            <Sparkles size={12} className="text-amber-400" />
          </button>
          <div className="h-px bg-white/5 my-1" />
        </>
      )}

      {/* Copy Selected Text */}
      {selectedText.trim().length > 0 && (
        <button
          onClick={handleCopy}
          className="w-full px-3 py-2 rounded-xl text-left font-medium text-white hover:bg-indigo-600/30 hover:text-indigo-200 border border-transparent hover:border-indigo-500/30 flex items-center gap-2.5 transition cursor-pointer active:scale-98"
        >
          {copied ? <Check size={14} className="text-emerald-400" /> : <Copy size={14} className="text-indigo-400" />}
          <span>{copied ? 'Copied to Clipboard!' : 'Copy Selection'}</span>
        </button>
      )}

      {/* Copy Entire Message */}
      {onCopyFullNotification && (
        <button
          onClick={() => {
            onCopyFullNotification();
            onClose();
          }}
          className="w-full px-3 py-2 rounded-xl text-left font-medium text-gray-300 hover:bg-white/10 hover:text-white flex items-center gap-2.5 transition cursor-pointer active:scale-98"
        >
          <Copy size={14} className="text-gray-400" />
          <span>Copy Full Message</span>
        </button>
      )}

      {/* Delete Item */}
      {onDeleteNotification && (
        <>
          <div className="h-px bg-white/5 my-1" />
          <button
            onClick={() => {
              onDeleteNotification();
              onClose();
            }}
            className="w-full px-3 py-2 rounded-xl text-left font-medium text-rose-400 hover:bg-rose-500/15 flex items-center gap-2.5 transition cursor-pointer active:scale-98"
          >
            <Trash2 size={14} />
            <span>Delete Notification</span>
          </button>
        </>
      )}
    </div>
  );
};
