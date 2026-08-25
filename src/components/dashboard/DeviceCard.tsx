import { BatteryCharging, Battery, HardDrive, Wifi, Smartphone } from 'lucide-react';
import { useDeviceStore } from '../../stores/deviceStore';
import { MediaControls } from '../controls/MediaControls';

export const DeviceCard: React.FC = () => {
  const { activeDevice, connectionState, telemetry, setPairingModalOpen, disconnect } = useDeviceStore();

  if (!activeDevice || connectionState === 'disconnected') {
    return (
      <div className="bg-[#181a20] border border-[#262933] rounded-2xl p-6 flex flex-col items-center justify-center text-center gap-4 py-12">
        <div className="w-14 h-14 rounded-full bg-indigo-600/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400">
          <Smartphone size={28} />
        </div>
        <div className="flex flex-col gap-1 max-w-sm">
          <h3 className="text-lg font-semibold text-white">No Device Connected</h3>
          <p className="text-xs text-gray-400">
            Pair with your Android device via Wireless Debugging to view notifications, battery telemetry, and quick controls.
          </p>
        </div>
        <button
          onClick={() => setPairingModalOpen(true)}
          className="mt-2 px-5 py-2.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-medium rounded-xl transition shadow-lg shadow-indigo-600/20 flex items-center gap-2 active:scale-95 cursor-pointer"
        >
          <Smartphone size={15} />
          Pair / Connect Device
        </button>
      </div>
    );
  }

  const getBatteryColor = (level: number, status?: string) => {
    if (status === 'charging') return 'text-emerald-400';
    if (level <= 20) return 'text-rose-400';
    if (level <= 50) return 'text-amber-400';
    return 'text-emerald-400';
  };

  return (
    <div className="flex flex-col gap-4">
      {/* Primary Device Header Banner */}
      <div className="bg-[#181a20] border border-[#262933] rounded-2xl p-5 flex items-center justify-between shadow-sm">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 border border-indigo-500/30 flex items-center justify-center text-indigo-400">
            <Smartphone size={24} />
          </div>
          <div className="flex flex-col">
            <div className="flex items-center gap-2">
              <h2 className="text-base font-bold text-white tracking-tight">{activeDevice.model}</h2>
              <span className="px-2 py-0.5 rounded-full text-[10px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                Connected
              </span>
            </div>
            <span className="text-xs text-gray-400 flex items-center gap-2 mt-0.5">
              <span>{activeDevice.manufacturer}</span>
              <span>•</span>
              <span>Android {activeDevice.android_version} (API {activeDevice.sdk_version})</span>
              <span>•</span>
              <span className="font-mono text-gray-500 text-[11px]">{activeDevice.serial}</span>
            </span>
          </div>
        </div>

        <button
          onClick={disconnect}
          className="px-3 py-1.5 bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/20 text-xs font-medium rounded-lg transition active:scale-95 cursor-pointer"
        >
          Disconnect
        </button>
      </div>

      {/* Telemetry Metrics Grid */}
      <div className="grid grid-cols-3 gap-3">
        {/* Battery Widget */}
        <div className="bg-[#181a20] border border-[#262933] rounded-xl p-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div
              className={`p-2.5 rounded-lg bg-[#111317] border border-[#20222a] ${getBatteryColor(
                telemetry?.battery_level ?? 100,
                telemetry?.battery_status,
              )}`}
            >
              {telemetry?.battery_status === 'charging' ? <BatteryCharging size={20} /> : <Battery size={20} />}
            </div>
            <div className="flex flex-col">
              <span className="text-xs text-gray-400">Battery</span>
              <span className="text-lg font-bold text-white leading-tight">
                {telemetry ? `${telemetry.battery_level}%` : '--'}
              </span>
            </div>
          </div>
          <div className="flex flex-col items-end text-[11px] text-gray-500">
            <span className="capitalize">{telemetry?.battery_status ?? 'Active'}</span>
            <span>{telemetry ? `${telemetry.battery_temp_celsius.toFixed(1)}°C` : ''}</span>
          </div>
        </div>

        {/* Storage Widget */}
        <div className="bg-[#181a20] border border-[#262933] rounded-xl p-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-lg bg-[#111317] border border-[#20222a] text-purple-400">
              <HardDrive size={20} />
            </div>
            <div className="flex flex-col">
              <span className="text-xs text-gray-400">Storage</span>
              <span className="text-lg font-bold text-white leading-tight">
                {telemetry ? `${telemetry.storage_free_gb.toFixed(1)} GB` : '--'}
              </span>
            </div>
          </div>
          <div className="flex flex-col items-end text-[11px] text-gray-500">
            <span>Free Space</span>
            <span>{telemetry ? `${telemetry.storage_used_percent}% used` : ''}</span>
          </div>
        </div>

        {/* Wi-Fi Widget */}
        <div className="bg-[#181a20] border border-[#262933] rounded-xl p-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2.5 rounded-lg bg-[#111317] border border-[#20222a] text-cyan-400">
              <Wifi size={20} />
            </div>
            <div className="flex flex-col">
              <span className="text-xs text-gray-400">Wi-Fi</span>
              <span className="text-sm font-semibold text-white leading-tight truncate max-w-[100px]">
                {telemetry?.wifi_ssid ?? 'Local Wi-Fi'}
              </span>
            </div>
          </div>
          <div className="flex flex-col items-end text-[11px] text-gray-500">
            <span>Wireless ADB</span>
            <span>{telemetry?.wifi_signal_dbm ? `${telemetry.wifi_signal_dbm} dBm` : 'Connected'}</span>
          </div>
        </div>
      </div>

      {/* Quick Remote Controls */}
      <MediaControls serial={activeDevice.serial} />
    </div>
  );
};
