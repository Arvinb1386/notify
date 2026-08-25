import { useMemo } from 'react';
import { Bell, Key, Copy, Check, Search, Trash2, Sparkles } from 'lucide-react';
import { useNotificationStore } from '../../stores/notificationStore';

export const NotificationsFeed: React.FC = () => {
  const {
    notifications,
    searchQuery,
    setSearchQuery,
    selectedPackage,
    setSelectedPackage,
    copiedOtp,
    copyOtp,
    clearHistory,
  } = useNotificationStore();

  // Unique apps for filtering
  const appFilters = useMemo(() => {
    const pkgs = new Set<string>();
    notifications.forEach((n) => pkgs.add(n.package_name));
    return Array.from(pkgs);
  }, [notifications]);

  // Filtered notifications
  const filteredNotifications = useMemo(() => {
    return notifications.filter((n) => {
      const matchSearch =
        searchQuery.trim() === '' ||
        (n.title && n.title.toLowerCase().includes(searchQuery.toLowerCase())) ||
        (n.body && n.body.toLowerCase().includes(searchQuery.toLowerCase())) ||
        (n.app_name && n.app_name.toLowerCase().includes(searchQuery.toLowerCase()));

      const matchPkg = !selectedPackage || n.package_name === selectedPackage;

      return matchSearch && matchPkg;
    });
  }, [notifications, searchQuery, selectedPackage]);

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div className="flex flex-col gap-4">
      {/* Search & Actions Header */}
      <div className="flex items-center justify-between gap-3">
        <div className="relative flex-1">
          <Search size={14} className="absolute left-3 top-2.5 text-gray-500" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search notifications or 2FA codes..."
            className="w-full bg-[#181a20] border border-[#262933] focus:border-indigo-500 rounded-xl pl-9 pr-3 py-2 text-xs text-white placeholder-gray-500 outline-hidden transition"
          />
        </div>

        {notifications.length > 0 && (
          <button
            onClick={clearHistory}
            className="p-2 hover:bg-rose-500/10 text-gray-400 hover:text-rose-400 border border-[#262933] hover:border-rose-500/30 rounded-xl transition cursor-pointer"
            title="Clear Notification History"
          >
            <Trash2 size={15} />
          </button>
        )}
      </div>

      {/* App Filter Pills */}
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

          {appFilters.map((pkg) => {
            const count = notifications.filter((n) => n.package_name === pkg).length;
            const appName = notifications.find((n) => n.package_name === pkg)?.app_name || pkg;
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
                {appName} ({count})
              </button>
            );
          })}
        </div>
      )}

      {/* Notifications List */}
      {filteredNotifications.length === 0 ? (
        <div className="bg-[#181a20] border border-[#262933] rounded-2xl p-8 flex flex-col items-center justify-center text-center gap-2 py-16">
          <div className="w-12 h-12 rounded-full bg-[#111317] border border-[#262933] flex items-center justify-center text-gray-500">
            <Bell size={22} />
          </div>
          <span className="text-sm font-semibold text-gray-300">No Notifications Received Yet</span>
          <p className="text-xs text-gray-500 max-w-xs">
            Incoming notifications, SMS alerts, and 2FA verification codes will appear here in real-time.
          </p>
        </div>
      ) : (
        <div className="flex flex-col gap-2.5">
          {filteredNotifications.map((item) => (
            <div
              key={item.id}
              className={`bg-[#181a20] border rounded-xl p-4 flex flex-col gap-2.5 transition relative overflow-hidden shadow-xs hover:border-[#353947] ${
                item.is_otp
                  ? 'border-indigo-500/40 bg-gradient-to-r from-indigo-950/20 via-[#181a20] to-[#181a20]'
                  : 'border-[#262933]'
              }`}
            >
              {/* Card Header */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-6 h-6 rounded-md bg-[#111317] border border-[#262933] flex items-center justify-center text-indigo-400 text-xs font-bold">
                    {item.app_name ? item.app_name[0].toUpperCase() : 'A'}
                  </div>
                  <span className="text-xs font-semibold text-white">
                    {item.app_name || item.package_name}
                  </span>
                </div>

                <div className="flex items-center gap-2">
                  {item.is_otp && (
                    <span className="px-2 py-0.5 rounded-md text-[10px] font-bold bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 flex items-center gap-1">
                      <Key size={10} /> 2FA / OTP
                    </span>
                  )}
                  <span className="text-[11px] text-gray-500 font-mono">
                    {formatTime(item.post_time)}
                  </span>
                </div>
              </div>

              {/* Title & Body */}
              <div className="flex flex-col gap-0.5">
                {item.title && (
                  <h4 className="text-xs font-semibold text-gray-200">{item.title}</h4>
                )}
                {item.body && (
                  <p className="text-xs text-gray-400 leading-relaxed break-words">{item.body}</p>
                )}
                {item.subtext && (
                  <span className="text-[11px] text-gray-500">{item.subtext}</span>
                )}
              </div>

              {/* 1-Click OTP Copy Action Banner */}
              {item.is_otp && item.otp_code && (
                <div className="mt-1 bg-indigo-950/40 border border-indigo-500/30 rounded-lg p-2.5 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Sparkles size={14} className="text-amber-400" />
                    <span className="text-xs text-gray-300">Verification Code:</span>
                    <span className="font-mono text-sm font-bold text-white tracking-wider bg-[#111317] px-2 py-0.5 rounded-md border border-indigo-500/30">
                      {item.otp_code}
                    </span>
                  </div>

                  <button
                    onClick={() => copyOtp(item.otp_code!)}
                    className={`px-3 py-1 text-xs font-semibold rounded-md transition flex items-center gap-1.5 cursor-pointer active:scale-95 ${
                      copiedOtp === item.otp_code
                        ? 'bg-emerald-600 text-white'
                        : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-xs'
                    }`}
                  >
                    {copiedOtp === item.otp_code ? (
                      <>
                        <Check size={13} /> Copied!
                      </>
                    ) : (
                      <>
                        <Copy size={13} /> Copy Code
                      </>
                    )}
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
