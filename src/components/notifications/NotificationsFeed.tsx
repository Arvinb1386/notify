import { useMemo, useState } from 'react';
import {
  Bell,
  Key,
  Copy,
  Check,
  Search,
  Trash2,
  Sparkles,
  Clock,
  ChevronDown,
  ChevronUp,
} from 'lucide-react';
import { useNotificationStore } from '../../stores/notificationStore';

export const NotificationsFeed: React.FC = () => {
  const {
    notifications,
    searchQuery,
    setSearchQuery,
    selectedPackage,
    setSelectedPackage,
    filterOtpOnly,
    setFilterOtpOnly,
    copiedOtp,
    copiedId,
    copyOtp,
    copyNotificationText,
    deleteNotification,
    clearHistory,
  } = useNotificationStore();

  const [expandedId, setExpandedId] = useState<string | null>(null);

  // Extract unique apps and sort alphabetically
  const appFilters = useMemo(() => {
    const pkgs = new Map<string, string>();
    notifications.forEach((n) => {
      pkgs.set(n.package_name, n.app_name || n.package_name);
    });
    return Array.from(pkgs.entries()).sort((a, b) => a[1].localeCompare(b[1]));
  }, [notifications]);

  // Filtered notifications
  const filteredNotifications = useMemo(() => {
    return notifications.filter((n) => {
      const matchSearch =
        searchQuery.trim() === '' ||
        (n.title && n.title.toLowerCase().includes(searchQuery.toLowerCase())) ||
        (n.body && n.body.toLowerCase().includes(searchQuery.toLowerCase())) ||
        (n.app_name && n.app_name.toLowerCase().includes(searchQuery.toLowerCase())) ||
        (n.otp_code && n.otp_code.includes(searchQuery.trim()));

      const matchPkg = !selectedPackage || n.package_name === selectedPackage;
      const matchOtp = !filterOtpOnly || n.is_otp;

      return matchSearch && matchPkg && matchOtp;
    });
  }, [notifications, searchQuery, selectedPackage, filterOtpOnly]);

  const formatFullDate = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleString([], {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  };

  const getAppColor = (pkg: string) => {
    if (pkg.includes('telegram') || pkg.includes('turbo') || pkg.includes('ellipi')) return 'text-sky-400 bg-sky-500/10 border-sky-500/20';
    if (pkg.includes('whatsapp')) return 'text-emerald-400 bg-emerald-500/10 border-emerald-500/20';
    if (pkg.includes('instagram')) return 'text-pink-400 bg-pink-500/10 border-pink-500/20';
    if (pkg.includes('eitaa')) return 'text-amber-400 bg-amber-500/10 border-amber-500/20';
    if (pkg.includes('bale')) return 'text-teal-400 bg-teal-500/10 border-teal-500/20';
    if (pkg.includes('messaging') || pkg.includes('sms')) return 'text-indigo-400 bg-indigo-500/10 border-indigo-500/20';
    if (pkg.includes('bank') || pkg.includes('blu')) return 'text-blue-400 bg-blue-500/10 border-blue-500/20';
    if (pkg.includes('digikala')) return 'text-red-400 bg-red-500/10 border-red-500/20';
    return 'text-gray-300 bg-gray-500/10 border-gray-500/20';
  };

  return (
    <div className="flex flex-col gap-4">
      {/* Search & Global Filter Bar */}
      <div className="flex items-center justify-between gap-2.5">
        <div className="relative flex-1">
          <Search size={14} className="absolute left-3 top-2.5 text-gray-500" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search notifications, messages or 2FA codes..."
            className="w-full bg-[#181a20] border border-[#262933] focus:border-indigo-500 rounded-xl pl-9 pr-3 py-2 text-xs text-white placeholder-gray-500 outline-hidden transition"
          />
        </div>

        {/* 2FA / OTP Filter Toggle */}
        <button
          onClick={() => setFilterOtpOnly(!filterOtpOnly)}
          className={`px-3 py-2 rounded-xl text-xs font-medium border flex items-center gap-1.5 transition cursor-pointer active:scale-95 ${
            filterOtpOnly
              ? 'bg-amber-500/20 border-amber-500/40 text-amber-300'
              : 'bg-[#181a20] border-[#262933] text-gray-400 hover:text-white'
          }`}
          title="Filter only 2FA & OTP verification codes"
        >
          <Key size={13} />
          <span>OTP Only</span>
        </button>

        {notifications.length > 0 && (
          <button
            onClick={clearHistory}
            className="p-2 hover:bg-rose-500/10 text-gray-400 hover:text-rose-400 border border-[#262933] hover:border-rose-500/30 rounded-xl transition cursor-pointer"
            title="Clear all notification history"
          >
            <Trash2 size={15} />
          </button>
        )}
      </div>

      {/* App Filter Chips */}
      {appFilters.length > 0 && (
        <div className="flex items-center gap-1.5 overflow-x-auto pb-1 scrollbar-none">
          <button
            onClick={() => setSelectedPackage(null)}
            className={`px-3 py-1 rounded-full text-xs font-medium transition whitespace-nowrap cursor-pointer ${
              selectedPackage === null
                ? 'bg-indigo-600 text-white shadow-xs'
                : 'bg-[#181a20] text-gray-400 hover:text-white border border-[#262933]'
            }`}
          >
            All Apps ({notifications.length})
          </button>

          {appFilters.map(([pkg, name]) => {
            const count = notifications.filter((n) => n.package_name === pkg).length;
            return (
              <button
                key={pkg}
                onClick={() => setSelectedPackage(pkg === selectedPackage ? null : pkg)}
                className={`px-3 py-1 rounded-full text-xs font-medium transition whitespace-nowrap cursor-pointer ${
                  selectedPackage === pkg
                    ? 'bg-indigo-600 text-white'
                    : 'bg-[#181a20] text-gray-400 hover:text-white border border-[#262933]'
                }`}
              >
                {name} ({count})
              </button>
            );
          })}
        </div>
      )}

      {/* Notifications Stream Feed */}
      {filteredNotifications.length === 0 ? (
        <div className="bg-[#181a20] border border-[#262933] rounded-2xl p-8 flex flex-col items-center justify-center text-center gap-2 py-16">
          <div className="w-12 h-12 rounded-full bg-[#111317] border border-[#262933] flex items-center justify-center text-gray-500">
            <Bell size={22} />
          </div>
          <span className="text-sm font-semibold text-gray-300">No Notifications Match Filter</span>
          <p className="text-xs text-gray-500 max-w-xs">
            {filterOtpOnly
              ? 'No 2FA / OTP verification codes received yet.'
              : 'Incoming phone notifications will appear here in real-time.'}
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-2.5">
          {filteredNotifications.map((item) => {
            const isExpanded = expandedId === item.id;
            const fullContent = [item.title, item.body, item.subtext].filter(Boolean).join('\n');

            return (
              <div
                key={item.id}
                className={`bg-[#181a20] border rounded-2xl p-4 flex flex-col gap-2.5 transition relative overflow-hidden shadow-xs hover:border-[#3a3e4e] ${
                  item.is_otp
                    ? 'border-indigo-500/40 bg-gradient-to-r from-indigo-950/25 via-[#181a20] to-[#181a20]'
                    : 'border-[#262933]'
                }`}
              >
                {/* Header info */}
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2.5">
                    <div
                      className={`w-7 h-7 rounded-lg border flex items-center justify-center text-xs font-bold ${getAppColor(
                        item.package_name
                      )}`}
                    >
                      {item.app_name ? item.app_name[0].toUpperCase() : 'A'}
                    </div>
                    <div className="flex flex-col">
                      <span className="text-xs font-bold text-white leading-none">
                        {item.app_name || item.package_name}
                      </span>
                      <span className="text-[10px] text-gray-500 font-mono mt-0.5">
                        {item.package_name}
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center gap-2">
                    {item.is_otp && (
                      <span className="px-2 py-0.5 rounded-md text-[10px] font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30 flex items-center gap-1">
                        <Key size={10} /> 2FA / OTP
                      </span>
                    )}
                    <span className="text-[11px] text-gray-400 font-mono flex items-center gap-1">
                      <Clock size={11} className="text-gray-500" />
                      {formatFullDate(item.post_time)}
                    </span>
                  </div>
                </div>

                {/* Body Content */}
                <div className="flex flex-col gap-1 mt-0.5">
                  {item.title && (
                    <h4 className="text-xs font-semibold text-gray-100">{item.title}</h4>
                  )}
                  {item.body && (
                    <p
                      className={`text-xs text-gray-300 leading-relaxed whitespace-pre-wrap ${
                        !isExpanded && item.body.length > 200 ? 'line-clamp-3' : ''
                      }`}
                    >
                      {item.body}
                    </p>
                  )}
                  {item.subtext && (
                    <span className="text-[11px] text-gray-400 font-medium">{item.subtext}</span>
                  )}
                </div>

                {/* 1-Click OTP Action Banner */}
                {item.is_otp && item.otp_code && (
                  <div className="mt-1 bg-gradient-to-r from-indigo-950/60 to-purple-950/40 border border-indigo-500/30 rounded-xl p-3 flex items-center justify-between shadow-inner">
                    <div className="flex items-center gap-2.5">
                      <Sparkles size={16} className="text-amber-400 animate-pulse" />
                      <div className="flex flex-col">
                        <span className="text-[11px] font-medium text-gray-300">Detected Security Code</span>
                        <span className="font-mono text-base font-extrabold text-white tracking-widest mt-0.5">
                          {item.otp_code}
                        </span>
                      </div>
                    </div>

                    <button
                      onClick={() => copyOtp(item.otp_code!)}
                      className={`px-4 py-2 text-xs font-bold rounded-xl transition flex items-center gap-1.5 cursor-pointer active:scale-95 shadow-md ${
                        copiedOtp === item.otp_code
                          ? 'bg-emerald-600 text-white'
                          : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-indigo-600/30'
                      }`}
                    >
                      {copiedOtp === item.otp_code ? (
                        <>
                          <Check size={14} /> Copied!
                        </>
                      ) : (
                        <>
                          <Copy size={14} /> Copy Code
                        </>
                      )}
                    </button>
                  </div>
                )}

                {/* Card Footer Quick Actions */}
                <div className="flex items-center justify-between border-t border-[#262933]/60 pt-2.5 mt-1 text-[11px] text-gray-400">
                  <div className="flex items-center gap-2">
                    {/* Copy entire notification text */}
                    <button
                      onClick={() => copyNotificationText(item.id, fullContent)}
                      className="px-2.5 py-1 hover:bg-[#262933] text-gray-400 hover:text-white rounded-lg transition flex items-center gap-1.5 cursor-pointer"
                      title="Copy full message text to clipboard"
                    >
                      {copiedId === item.id ? (
                        <>
                          <Check size={12} className="text-emerald-400" /> Copied Text
                        </>
                      ) : (
                        <>
                          <Copy size={12} /> Copy Text
                        </>
                      )}
                    </button>

                    {item.body && item.body.length > 200 && (
                      <button
                        onClick={() => setExpandedId(isExpanded ? null : item.id)}
                        className="px-2.5 py-1 hover:bg-[#262933] text-gray-400 hover:text-white rounded-lg transition flex items-center gap-1 cursor-pointer"
                      >
                        {isExpanded ? (
                          <>
                            <ChevronUp size={12} /> Show Less
                          </>
                        ) : (
                          <>
                            <ChevronDown size={12} /> Expand
                          </>
                        )}
                      </button>
                    )}
                  </div>

                  {/* Delete individual notification */}
                  <button
                    onClick={() => deleteNotification(item.id)}
                    className="p-1 hover:bg-rose-500/10 text-gray-500 hover:text-rose-400 rounded-md transition cursor-pointer"
                    title="Delete notification"
                  >
                    <Trash2 size={13} />
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};
