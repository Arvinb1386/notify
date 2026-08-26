import { useState, useEffect } from 'react';
import {
  Smartphone,
  Bell,
  Activity,
  Radio,
  Plus,
  Layers,
} from 'lucide-react';
import { useDeviceStore } from './stores/deviceStore';
import { useNotificationStore } from './stores/notificationStore';
import { DeviceCard } from './components/dashboard/DeviceCard';
import { DevicesTab } from './components/dashboard/DevicesTab';
import { NotificationsFeed } from './components/notifications/NotificationsFeed';
import { PairingWizard } from './components/pairing/PairingWizard';
import { tauriApi } from './lib/ipc';

export function App() {
  const [activeTab, setActiveTab] = useState<'dashboard' | 'devices' | 'notifications' | 'diagnostics'>('dashboard');
  const [adbVersion, setAdbVersion] = useState<string>('Detecting...');

  const {
    activeDevice,
    connectionState,
    savedDevices,
    initConnectionListeners,
    setPairingModalOpen,
  } = useDeviceStore();

  const { notifications, initNotificationListener } = useNotificationStore();

  useEffect(() => {
    let unlistenConn: (() => void) | undefined;
    let unlistenNotif: (() => void) | undefined;

    const setup = async () => {
      try {
        const ver = await tauriApi.checkAdbStatus();
        setAdbVersion(ver.split('\n')[0] || 'ADB Ready');
      } catch (e) {
        setAdbVersion('ADB Not Found');
      }

      unlistenConn = await initConnectionListeners();
      unlistenNotif = await initNotificationListener();
    };

    setup();

    return () => {
      if (unlistenConn) unlistenConn();
      if (unlistenNotif) unlistenNotif();
    };
  }, []);

  const getStatusBadge = () => {
    switch (connectionState) {
      case 'connected':
        return (
          <span className="px-2.5 py-1 rounded-full text-[11px] font-medium bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
            Connected
          </span>
        );
      case 'connecting':
      case 'discovering':
        return (
          <span className="px-2.5 py-1 rounded-full text-[11px] font-medium bg-amber-500/10 text-amber-400 border border-amber-500/20 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-amber-400 animate-spin"></span>
            {connectionState === 'discovering' ? 'Searching Device...' : 'Connecting...'}
          </span>
        );
      case 'degraded':
        return (
          <span className="px-2.5 py-1 rounded-full text-[11px] font-medium bg-rose-500/10 text-rose-400 border border-rose-500/20 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-rose-400"></span>
            Degraded
          </span>
        );
      default:
        return (
          <span className="px-2.5 py-1 rounded-full text-[11px] font-medium bg-gray-500/10 text-gray-400 border border-gray-500/20 flex items-center gap-1.5">
            <span className="w-2 h-2 rounded-full bg-gray-500"></span>
            Offline
          </span>
        );
    }
  };

  return (
    <div className="min-h-screen bg-[#0f1117] text-gray-200 flex flex-col font-sans selection:bg-indigo-600 selection:text-white">
      {/* Top Application Bar */}
      <header className="h-14 border-b border-[#20222a] bg-[#14161d] px-6 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-xl bg-gradient-to-tr from-indigo-600 to-purple-600 flex items-center justify-center text-white shadow-md shadow-indigo-600/30">
            <Radio size={18} />
          </div>
          <div>
            <h1 className="text-sm font-bold text-white tracking-tight leading-none">Notify</h1>
            <span className="text-[10px] text-gray-400 font-medium">Wireless ADB Companion</span>
          </div>
        </div>

        {/* Global Connection Status Badge */}
        <div className="flex items-center gap-3">
          {getStatusBadge()}

          {(!activeDevice || connectionState === 'disconnected') && (
            <button
              onClick={() => setPairingModalOpen(true)}
              className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold rounded-lg transition shadow-xs flex items-center gap-1.5 cursor-pointer active:scale-95"
            >
              <Plus size={14} /> Connect Device
            </button>
          )}
        </div>
      </header>

      {/* Main App Layout */}
      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar Navigation */}
        <aside className="w-56 border-r border-[#20222a] bg-[#14161d] p-4 flex flex-col justify-between shrink-0">
          <div className="flex flex-col gap-1">
            <button
              onClick={() => setActiveTab('dashboard')}
              className={`w-full px-3 py-2.5 rounded-xl text-xs font-medium flex items-center gap-2.5 transition cursor-pointer ${
                activeTab === 'dashboard'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-gray-400 hover:text-gray-200 hover:bg-[#1c1e27]'
              }`}
            >
              <Smartphone size={16} />
              <span>Device Dashboard</span>
            </button>

            <button
              onClick={() => setActiveTab('devices')}
              className={`w-full px-3 py-2.5 rounded-xl text-xs font-medium flex items-center justify-between transition cursor-pointer ${
                activeTab === 'devices'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-gray-400 hover:text-gray-200 hover:bg-[#1c1e27]'
              }`}
            >
              <div className="flex items-center gap-2.5">
                <Layers size={16} />
                <span>Devices</span>
              </div>
              {savedDevices.length > 0 && (
                <span className="px-1.5 py-0.5 rounded-full text-[10px] font-bold bg-white/10 text-gray-300">
                  {savedDevices.length}
                </span>
              )}
            </button>

            <button
              onClick={() => setActiveTab('notifications')}
              className={`w-full px-3 py-2.5 rounded-xl text-xs font-medium flex items-center justify-between transition cursor-pointer ${
                activeTab === 'notifications'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-gray-400 hover:text-gray-200 hover:bg-[#1c1e27]'
              }`}
            >
              <div className="flex items-center gap-2.5">
                <Bell size={16} />
                <span>Notifications</span>
              </div>
              {notifications.length > 0 && (
                <span className="px-1.5 py-0.5 rounded-full text-[10px] font-bold bg-white/20 text-white">
                  {notifications.length}
                </span>
              )}
            </button>

            <button
              onClick={() => setActiveTab('diagnostics')}
              className={`w-full px-3 py-2.5 rounded-xl text-xs font-medium flex items-center gap-2.5 transition cursor-pointer ${
                activeTab === 'diagnostics'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-gray-400 hover:text-gray-200 hover:bg-[#1c1e27]'
              }`}
            >
              <Activity size={16} />
              <span>System Health</span>
            </button>
          </div>

          {/* Bottom Sidebar Info */}
          <div className="bg-[#181a20] border border-[#262933] rounded-xl p-3 flex flex-col gap-1 text-[11px] text-gray-400">
            <div className="flex items-center justify-between text-gray-300 font-medium">
              <span>Pure ADB</span>
              <span className="text-[10px] text-emerald-400">Zero-Install</span>
            </div>
            <span className="text-gray-500 font-mono text-[10px] truncate">{adbVersion}</span>
          </div>
        </aside>

        {/* Content Area */}
        <main className="flex-1 overflow-y-auto p-6 bg-[#0f1117]">
          <div className="max-w-3xl mx-auto flex flex-col gap-6">
            {activeTab === 'dashboard' && <DeviceCard />}
            {activeTab === 'devices' && <DevicesTab />}
            {activeTab === 'notifications' && <NotificationsFeed />}
            {activeTab === 'diagnostics' && (
              <div className="bg-[#181a20] border border-[#262933] rounded-2xl p-6 flex flex-col gap-4">
                <div className="flex items-center gap-2 text-white font-bold text-sm">
                  <Activity size={18} className="text-indigo-400" />
                  <span>Subsystem Diagnostics & Health</span>
                </div>
                <div className="grid grid-cols-2 gap-3 text-xs">
                  <div className="bg-[#111317] border border-[#262933] p-3 rounded-xl flex flex-col gap-1">
                    <span className="text-gray-400">ADB Transport</span>
                    <span className="font-semibold text-emerald-400">● Operational</span>
                  </div>
                  <div className="bg-[#111317] border border-[#262933] p-3 rounded-xl flex flex-col gap-1">
                    <span className="text-gray-400">Notification Engine</span>
                    <span className="font-semibold text-emerald-400">● Hybrid Logcat + Dumpsys</span>
                  </div>
                  <div className="bg-[#111317] border border-[#262933] p-3 rounded-xl flex flex-col gap-1">
                    <span className="text-gray-400">OTP Extractor (EN/FA/AR)</span>
                    <span className="font-semibold text-emerald-400">● Confidence Scorer Active</span>
                  </div>
                  <div className="bg-[#111317] border border-[#262933] p-3 rounded-xl flex flex-col gap-1">
                    <span className="text-gray-400">Database Storage</span>
                    <span className="font-semibold text-emerald-400">● SQLite Migrated</span>
                  </div>
                </div>
              </div>
            )}
          </div>
        </main>
      </div>

      {/* Pairing & Connection Modal */}
      <PairingWizard />
    </div>
  );
}

export default App;
