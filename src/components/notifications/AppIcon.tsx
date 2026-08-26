import React from 'react';

interface AppIconProps {
  packageName: string;
  appName?: string | null;
  size?: number;
  className?: string;
}

export const AppIcon: React.FC<AppIconProps> = ({
  packageName,
  appName,
  size = 28,
  className = '',
}) => {
  const pkg = packageName.toLowerCase();

  // Telegram / TurboTel / Mobogram
  if (pkg.includes('telegram') || pkg.includes('turbo') || pkg.includes('ellipi') || pkg.includes('nasim')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-gradient-to-tr from-[#229ED9] to-[#38a9e0] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'Telegram / TurboTel'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69a.2.2 0 00-.05-.18c-.06-.05-.14-.03-.21-.02-.09.02-1.49.95-4.22 2.79-.4.27-.76.41-1.08.4-.36-.01-1.04-.2-1.55-.37-.63-.2-1.12-.31-1.08-.66.02-.18.27-.36.75-.55 2.92-1.27 4.86-2.11 5.83-2.51 2.78-1.16 3.35-1.36 3.73-1.36.08 0 .27.02.39.12.1.08.13.19.14.27-.01.06.01.24 0 .37z" />
        </svg>
      </div>
    );
  }

  // WhatsApp
  if (pkg.includes('whatsapp')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-[#25D366] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'WhatsApp'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M12.04 2c-5.46 0-9.91 4.45-9.91 9.91 0 1.75.46 3.45 1.32 4.95L2.05 22l5.25-1.38c1.45.79 3.08 1.21 4.74 1.21 5.46 0 9.91-4.45 9.91-9.91 0-2.65-1.03-5.14-2.9-7.01A9.816 9.816 0 0012.04 2zm0 18.12c-1.49 0-2.95-.4-4.23-1.16l-.3-.18-3.14.82.84-3.06-.2-.31c-.83-1.33-1.27-2.88-1.27-4.47 0-4.51 3.67-8.18 8.18-8.18 2.19 0 4.24.85 5.79 2.4 1.55 1.55 2.4 3.6 2.4 5.79 0 4.51-3.67 8.18-8.18 8.18zm4.49-6.13c-.25-.12-1.47-.72-1.7-.81-.23-.08-.39-.12-.56.12-.17.25-.64.81-.79.97-.14.17-.29.19-.54.06-.25-.12-1.05-.39-2-1.23-.74-.66-1.24-1.47-1.38-1.72-.14-.25-.02-.38.11-.5.11-.11.25-.29.37-.43.12-.14.17-.25.25-.41.08-.17.04-.31-.02-.43s-.56-1.34-.76-1.84c-.2-.48-.41-.42-.56-.43h-.48c-.17 0-.43.06-.66.31-.22.25-.87.85-.87 2.07s.89 2.4 1.01 2.57c.12.17 1.75 2.67 4.24 3.74.59.26 1.05.41 1.41.53.6.19 1.14.16 1.57.1.48-.07 1.47-.6 1.68-1.18.21-.58.21-1.07.14-1.18-.06-.1-.23-.17-.48-.29z" />
        </svg>
      </div>
    );
  }

  // Instagram / Threads
  if (pkg.includes('instagram') || pkg.includes('barcelona')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-gradient-to-tr from-[#feda75] via-[#d62976] to-[#4f5bd5] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'Instagram'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M12 2.163c3.204 0 3.584.012 4.85.07 3.252.148 4.771 1.691 4.919 4.919.058 1.265.069 1.645.069 4.849 0 3.205-.012 3.584-.069 4.849-.149 3.225-1.664 4.771-4.919 4.919-1.266.058-1.644.07-4.85.07-3.204 0-3.584-.012-4.849-.07-3.26-.149-4.771-1.699-4.919-4.92-.058-1.265-.07-1.644-.07-4.849 0-3.204.013-3.583.07-4.849.149-3.227 1.664-4.771 4.919-4.919 1.266-.057 1.645-.069 4.849-.069zm0-2.163c-3.259 0-3.667.014-4.947.072-4.358.2-6.78 2.618-6.98 6.98-.059 1.281-.073 1.689-.073 4.948 0 3.259.014 3.668.072 4.948.2 4.358 2.618 6.78 6.98 6.98 1.281.058 1.689.072 4.948.072 3.259 0 3.668-.014 4.948-.072 4.354-.2 6.782-2.618 6.979-6.98.059-1.28.073-1.689.073-4.948 0-3.259-.014-3.667-.072-4.947-.196-4.354-2.617-6.78-6.979-6.98-1.281-.059-1.69-.073-4.949-.073zm0 5.838c-3.403 0-6.162 2.759-6.162 6.162s2.759 6.163 6.162 6.163 6.162-2.759 6.162-6.163c0-3.403-2.759-6.162-6.162-6.162zm0 10.162c-2.209 0-4-1.79-4-4 0-2.209 1.791-4 4-4s4 1.791 4 4c0 2.21-1.791 4-4 4zm6.406-11.845c-.796 0-1.441.645-1.441 1.44s.645 1.44 1.441 1.44c.795 0 1.439-.645 1.439-1.44s-.644-1.44-1.439-1.44z" />
        </svg>
      </div>
    );
  }

  // Eitaa
  if (pkg.includes('eitaa')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-gradient-to-tr from-[#e67e22] to-[#f39c12] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 font-bold ${className}`}
        title={appName || 'Eitaa'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z" />
        </svg>
      </div>
    );
  }

  // Bale Messenger
  if (pkg.includes('bale')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-[#00a884] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'Bale'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M20 2H4c-1.1 0-1.99.9-1.99 2L2 22l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm-2 12H6v-2h12v2zm0-3H6V9h12v2zm0-3H6V6h12v2z" />
        </svg>
      </div>
    );
  }

  // Rubika
  if (pkg.includes('rubika') || pkg.includes('resaneh')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-gradient-to-tr from-[#9b59b6] to-[#e91e63] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'Rubika'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" stroke="currentColor" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round"/>
        </svg>
      </div>
    );
  }

  // Messages / SMS
  if (pkg.includes('messaging') || pkg.includes('mms') || pkg.includes('sms')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-gradient-to-tr from-[#1a73e8] to-[#4285f4] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'Messages'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M20 2H4c-1.1 0-2 .9-2 2v18l4-4h14c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H5.17L4 17.17V4h16v12zM7 9h2v2H7zm4 0h2v2h-2zm4 0h2v2h-2z" />
        </svg>
      </div>
    );
  }

  // Gmail / Outlook / Email
  if (pkg.includes('gm') || pkg.includes('gmail') || pkg.includes('mail') || pkg.includes('outlook')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-gradient-to-tr from-[#EA4335] to-[#f44336] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'Gmail'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z" />
        </svg>
      </div>
    );
  }

  // YouTube / YouTube Music
  if (pkg.includes('youtube')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-[#FF0000] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'YouTube'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M21.58 7.19c-.23-.86-.91-1.54-1.77-1.77C18.25 5 12 5 12 5s-6.25 0-7.81.42c-.86.23-1.54.91-1.77 1.77C2 8.75 2 12 2 12s0 3.25.42 4.81c.23.86.91 1.54 1.77 1.77C5.75 19 12 19 12 19s6.25 0 7.81-.42c.86-.23 1.54-.91 1.77-1.77C22 15.25 22 12 22 12s0-3.25-.42-4.81zM10 15V9l5.2 3-5.2 3z" />
        </svg>
      </div>
    );
  }

  // Spotify
  if (pkg.includes('spotify')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-[#1DB954] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'Spotify'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M12 2C6.4 2 2 6.4 2 12s4.4 10 10 10 10-4.4 10-10S17.6 2 12 2zm4.6 14.4c-.2.3-.5.4-.8.2-2.3-1.4-5.2-1.7-8.6-.9-.3.1-.7-.1-.8-.4-.1-.3.1-.7.4-.8 3.7-.8 6.9-.5 9.5 1.1.4.2.5.5.3.8zm1.2-2.7c-.2.4-.7.5-1.1.3-2.6-1.6-6.6-2.1-9.7-1.1-.4.1-.9-.1-1-.5-.1-.4.1-.9.5-1 3.6-1.1 8-.5 11 1.3.4.2.5.7.3 1zm.1-2.9C14.8 9 9.1 8.8 5.7 9.8c-.5.2-1.1-.1-1.2-.6s.1-1.1.6-1.2c3.9-1.2 10.1-.9 14 1.4.5.3.6.9.3 1.4-.2.4-.8.6-1.5.4z" />
        </svg>
      </div>
    );
  }

  // Twitter / X
  if (pkg.includes('twitter')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-black border border-[#2a2e39] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'X (Twitter)'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z" />
        </svg>
      </div>
    );
  }

  // Discord
  if (pkg.includes('discord')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-[#5865F2] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'Discord'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M20.317 4.37a19.791 19.791 0 00-4.885-1.515.074.074 0 00-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 00-5.487 0 12.64 12.64 0 00-.617-1.25.077.077 0 00-.079-.037A19.736 19.736 0 003.677 4.37a.07.07 0 00-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 00.031.057 19.9 19.9 0 005.993 3.03.078.078 0 00.084-.028c.462-.63.874-1.295 1.226-1.994.021-.041.001-.09-.041-.106a13.107 13.107 0 01-1.872-.892.077.077 0 01-.008-.128c.126-.093.252-.19.372-.287a.075.075 0 01.077-.01c3.929 1.793 8.18 1.793 12.061 0a.074.074 0 01.078.01c.12.098.246.194.373.288a.077.077 0 01-.006.127c-.598.35-1.22.648-1.873.891-.041.016-.062.066-.041.107.36.698.772 1.362 1.225 1.993a.076.076 0 00.084.028 19.839 19.839 0 006.002-3.03.077.077 0 00.032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 00-.031-.028zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z" />
        </svg>
      </div>
    );
  }

  // Digikala
  if (pkg.includes('digikala')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-[#ed1b34] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 font-black text-[11px] ${className}`}
        title={appName || 'Digikala'}
      >
        <span className="leading-none tracking-tighter">DK</span>
      </div>
    );
  }

  // Divar
  if (pkg.includes('divar')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-[#a62626] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 font-bold ${className}`}
        title={appName || 'Divar'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z" />
        </svg>
      </div>
    );
  }

  // Snapp
  if (pkg.includes('snapp')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-[#00d170] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 font-black text-xs ${className}`}
        title={appName || 'Snapp'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M18.92 6.01C18.72 5.42 18.16 5 17.5 5h-11c-.66 0-1.21.42-1.42 1.01L3 12v8c0 .55.45 1 1 1h1c.55 0 1-.45 1-1v-1h12v1c0 .55.45 1 1 1h1c.55 0 1-.45 1-1v-8l-2.08-5.99zM6.5 16c-.83 0-1.5-.67-1.5-1.5S5.67 13 6.5 13s1.5.67 1.5 1.5S7.33 16 6.5 16zm11 0c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5zM5 11l1.5-4.5h11L19 11H5z"/>
        </svg>
      </div>
    );
  }

  // Tapsi
  if (pkg.includes('tapsi')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-[#ff5722] flex items-center justify-center p-1 shadow-sm text-white shrink-0 font-black text-sm ${className}`}
        title={appName || 'Tapsi'}
      >
        <span>T</span>
      </div>
    );
  }

  // Banks (BluBank, Tejarat, Mellat, etc.)
  if (pkg.includes('bank') || pkg.includes('blu') || pkg.includes('saman')) {
    return (
      <div
        style={{ width: size, height: size }}
        className={`rounded-xl bg-gradient-to-tr from-[#0052cc] to-[#2684ff] flex items-center justify-center p-1.5 shadow-sm text-white shrink-0 ${className}`}
        title={appName || 'Banking App'}
      >
        <svg viewBox="0 0 24 24" fill="currentColor" className="w-full h-full">
          <path d="M4 10v7h3v-7H4zm6 0v7h3v-7h-3zM2 22h19v-3H2v3zm14-12v7h3v-7h-3zm-4.5-9L2 6v2h19V6l-9.5-5z" />
        </svg>
      </div>
    );
  }

  // Default Stylish App Icon Fallback with initials & colorful badge
  const initial = appName && appName.trim().length > 0 ? appName.trim()[0].toUpperCase() : pkg.slice(0, 1).toUpperCase();
  return (
    <div
      style={{ width: size, height: size }}
      className={`rounded-xl bg-gradient-to-tr from-[#2a2e3d] to-[#3a3f52] border border-[#444a5e] flex items-center justify-center text-xs font-bold text-white shadow-sm shrink-0 ${className}`}
      title={appName || packageName}
    >
      <span>{initial}</span>
    </div>
  );
};
