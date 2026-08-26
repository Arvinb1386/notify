import { Smartphone, Trash2, Power, Clock, Plus, CheckCircle2 } from 'lucide-react';
import { useDeviceStore } from '../../stores/deviceStore';

export const DevicesTab: React.FC = () => {
  const {
    savedDevices,
    activeDevice,
    connectionState,
    connectSavedDevice,
    deleteSavedDevice,
    setPairingModalOpen,
  } = useDeviceStore();

  const formatLastSeen = (ts: number) => {
    const diff = Date.now() - ts;
    const mins = Math.floor(diff / (1000 * 60));
    if (mins < 1) return 'Just now';
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    return `${Math.floor(hours / 24)}d ago`;
  };

  return (
    <div className="flex flex-col gap-5">
      {/* Header Banner */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-base font-bold text-white tracking-tight">Paired Devices</h2>
          <p className="text-xs text-gray-400">Connect to previously paired devices with a single click</p>
        </div>
        <button
          onClick={() => setPairingModalOpen(true)}
          className="px-3.5 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl transition shadow-sm flex items-center gap-1.5 cursor-pointer active:scale-95"
        >
          <Plus size={15} /> Add New Device
        </button>
      </div>

      {/* Devices List */}
      {savedDevices.length === 0 ? (
        <div className="bg-[#181a20] border border-[#262933] rounded-2xl p-8 flex flex-col items-center justify-center text-center gap-3 py-16">
          <div className="w-12 h-12 rounded-full bg-[#111317] border border-[#262933] flex items-center justify-center text-indigo-400">
            <Smartphone size={22} />
          </div>
          <div className="flex flex-col gap-1">
            <span className="text-sm font-semibold text-gray-300">No Saved Devices</span>
            <p className="text-xs text-gray-500 max-w-xs">
              When you pair and connect a phone via Wireless Debugging, it will be saved here for quick 1-click reconnects.
            </p>
          </div>
          <button
            onClick={() => setPairingModalOpen(true)}
            className="mt-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium rounded-xl transition cursor-pointer"
          >
            Pair Your First Device
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-3">
          {savedDevices.map((dev) => {
            const isCurrentlyConnected =
              activeDevice?.serial === dev.serial && connectionState === 'connected';

            return (
              <div
                key={dev.serial}
                className={`bg-[#181a20] border rounded-2xl p-4 flex items-center justify-between transition ${
                  isCurrentlyConnected
                    ? 'border-indigo-500/50 bg-gradient-to-r from-indigo-950/20 via-[#181a20] to-[#181a20]'
                    : 'border-[#262933] hover:border-[#353947]'
                }`}
              >
                <div className="flex items-center gap-3.5">
                  <div
                    className={`w-11 h-11 rounded-xl flex items-center justify-center border ${
                      isCurrentlyConnected
                        ? 'bg-indigo-600/20 border-indigo-500/40 text-indigo-400'
                        : 'bg-[#111317] border-[#262933] text-gray-400'
                    }`}
                  >
                    <Smartphone size={22} />
                  </div>

                  <div className="flex flex-col">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-bold text-white">{dev.model}</span>
                      {isCurrentlyConnected && (
                        <span className="px-2 py-0.5 rounded-full text-[10px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1">
                          <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                          Active
                        </span>
                      )}
                    </div>
                    <span className="text-xs text-gray-400 flex items-center gap-2 mt-0.5">
                      <span>{dev.manufacturer}</span>
                      <span>•</span>
                      <span>Android {dev.android_version}</span>
                      <span>•</span>
                      <span className="font-mono text-gray-500 text-[11px]">{dev.serial}</span>
                    </span>
                  </div>
                </div>

                <div className="flex items-center gap-2.5">
                  <span className="text-[11px] text-gray-500 flex items-center gap-1 font-mono">
                    <Clock size={11} /> {formatLastSeen(dev.last_connected)}
                  </span>

                  {!isCurrentlyConnected ? (
                    <button
                      onClick={() => connectSavedDevice(dev.serial)}
                      className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-xl transition shadow-xs flex items-center gap-1.5 cursor-pointer active:scale-95"
                    >
                      <Power size={13} /> Connect
                    </button>
                  ) : (
                    <div className="px-3 py-1.5 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-xs font-medium rounded-xl flex items-center gap-1.5">
                      <CheckCircle2 size={13} /> Connected
                    </div>
                  )}

                  <button
                    onClick={() => deleteSavedDevice(dev.serial)}
                    className="p-2 hover:bg-rose-500/10 text-gray-500 hover:text-rose-400 border border-[#262933] hover:border-rose-500/30 rounded-xl transition cursor-pointer"
                    title="Forget Device"
                  >
                    <Trash2 size={15} />
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
