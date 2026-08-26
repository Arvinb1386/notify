import { useState, useEffect } from 'react';
import { X, Wifi, KeyRound, ArrowRight, RefreshCw, CheckCircle2, AlertCircle, Sparkles, QrCode, Smartphone } from 'lucide-react';
import { QRCodeSVG } from 'qrcode.react';
import { useDeviceStore } from '../../stores/deviceStore';
import { tauriApi } from '../../lib/ipc';
import { DiscoveredService, PairingQrData } from '../../types';

export const PairingWizard: React.FC = () => {
  const { isPairingModalOpen, setPairingModalOpen, setActiveDevice, setConnectionState } = useDeviceStore();

  const [activeTab, setActiveTab] = useState<'companion' | 'pair' | 'connect'>('companion');
  const [host, setHost] = useState('192.168.1.');
  const [port, setPort] = useState('');
  const [code, setCode] = useState('');
  const [loading, setLoading] = useState(false);
  const [statusMsg, setStatusMsg] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);
  const [discoveredServices, setDiscoveredServices] = useState<DiscoveredService[]>([]);
  const [scanning, setScanning] = useState(false);
  const [qrData, setQrData] = useState<PairingQrData | null>(null);

  useEffect(() => {
    if (isPairingModalOpen) {
      handleScanMdns();
      fetchQrData();
    }
  }, [isPairingModalOpen]);

  const fetchQrData = async () => {
    try {
      const data = await tauriApi.getCompanionPairingQr();
      setQrData(data);
    } catch (e) {
      console.debug('Failed to get QR pairing data:', e);
    }
  };

  if (!isPairingModalOpen) return null;

  const handleScanMdns = async () => {
    setScanning(true);
    try {
      const services = await tauriApi.scanMdns();
      setDiscoveredServices(services);
    } catch (e) {
      console.debug('mDNS scan error:', e);
    } finally {
      setScanning(false);
    }
  };

  const handlePair = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!host || !port || !code) {
      setStatusMsg({ type: 'error', text: 'Please fill in IP, pairing port, and 6-digit code.' });
      return;
    }

    setLoading(true);
    setStatusMsg({ type: 'info', text: 'Pairing with Android device...' });

    try {
      const res = await tauriApi.pairDevice(host.trim(), parseInt(port), code.trim());
      setStatusMsg({ type: 'success', text: typeof res === 'string' ? res : 'Successfully paired!' });
      setTimeout(() => {
        setActiveTab('connect');
        setStatusMsg({ type: 'info', text: 'Now enter the Connect Port from your phone.' });
      }, 1200);
    } catch (err: any) {
      console.error('Pairing error detail:', err);
      const message =
        typeof err === 'string'
          ? err
          : err?.message || err?.code || JSON.stringify(err) || 'Pairing failed. Make sure phone and PC are on the same Wi-Fi.';
      setStatusMsg({
        type: 'error',
        text: message,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleConnect = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!host || !port) {
      setStatusMsg({ type: 'error', text: 'Please fill in IP and connect port.' });
      return;
    }

    setLoading(true);
    setStatusMsg({ type: 'info', text: `Connecting to ${host}:${port}...` });

    try {
      const dev = await tauriApi.connectDevice(host.trim(), parseInt(port));
      setStatusMsg({ type: 'success', text: `Connected to ${dev.model}!` });
      setActiveDevice(dev);
      setConnectionState('connected');
      setTimeout(() => {
        setPairingModalOpen(false);
      }, 1000);
    } catch (err: any) {
      console.error('Connection error detail:', err);
      const message =
        typeof err === 'string'
          ? err
          : err?.message || err?.code || JSON.stringify(err) || 'Connection failed. Verify the IP and port.';
      setStatusMsg({
        type: 'error',
        text: message,
      });
    } finally {
      setLoading(false);
    }
  };

  const selectDiscovered = (service: DiscoveredService) => {
    setHost(service.host);
    setPort(service.port.toString());
    if (service.service_type.includes('pairing')) {
      setActiveTab('pair');
    } else {
      setActiveTab('connect');
    }
  };

  const qrString = qrData
    ? `notify://pair?ip=${qrData.server_ip}&port=${qrData.port}&secret=${qrData.secret_token}`
    : 'notify://pair';

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-xs p-4 animate-in fade-in duration-200">
      <div className="bg-[#14161d] border border-[#262933] w-full max-w-lg rounded-2xl shadow-2xl overflow-hidden flex flex-col">
        {/* Header */}
        <div className="px-6 py-4 border-b border-[#262933] flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <div className="p-2 rounded-lg bg-indigo-600/10 text-indigo-400 border border-indigo-500/20">
              <Smartphone size={18} />
            </div>
            <div>
              <h3 className="text-sm font-bold text-white">Connect Device</h3>
              <p className="text-[11px] text-gray-400">Companion App (Instant) or ADB Wireless</p>
            </div>
          </div>
          <button
            onClick={() => setPairingModalOpen(false)}
            className="p-1.5 text-gray-400 hover:text-white rounded-lg hover:bg-[#262933] transition cursor-pointer"
          >
            <X size={18} />
          </button>
        </div>

        {/* Tab Switcher */}
        <div className="flex border-b border-[#262933] bg-[#111317]">
          <button
            onClick={() => {
              setActiveTab('companion');
              setStatusMsg(null);
            }}
            className={`flex-1 py-2.5 text-xs font-medium transition border-b-2 flex items-center justify-center gap-1.5 cursor-pointer ${
              activeTab === 'companion'
                ? 'border-indigo-500 text-indigo-400 bg-indigo-500/5 font-bold'
                : 'border-transparent text-gray-400 hover:text-gray-200'
            }`}
          >
            <QrCode size={13} />
            <span>Companion App (Fastest)</span>
          </button>

          <button
            onClick={() => {
              setActiveTab('pair');
              setStatusMsg(null);
            }}
            className={`flex-1 py-2.5 text-xs font-medium transition border-b-2 cursor-pointer ${
              activeTab === 'pair'
                ? 'border-indigo-500 text-indigo-400 bg-indigo-500/5'
                : 'border-transparent text-gray-400 hover:text-gray-200'
            }`}
          >
            ADB Wireless Pair
          </button>

          <button
            onClick={() => {
              setActiveTab('connect');
              setStatusMsg(null);
            }}
            className={`flex-1 py-2.5 text-xs font-medium transition border-b-2 cursor-pointer ${
              activeTab === 'connect'
                ? 'border-indigo-500 text-indigo-400 bg-indigo-500/5'
                : 'border-transparent text-gray-400 hover:text-gray-200'
            }`}
          >
            Manual Connect
          </button>
        </div>

        {/* Form Body */}
        <div className="p-6 flex flex-col gap-4">
          {activeTab === 'companion' ? (
            <div className="flex flex-col items-center gap-4 text-center">
              <div className="p-3 bg-white rounded-2xl shadow-lg border border-white/20">
                <QRCodeSVG value={qrString} size={160} />
              </div>

              <div className="flex flex-col gap-1 max-w-sm">
                <span className="text-xs font-bold text-white flex items-center justify-center gap-1.5">
                  <Sparkles size={14} className="text-amber-400" /> Auto-Discovery & Zero Configuration
                </span>
                <p className="text-[11px] text-gray-400">
                  Open the <b>Notify Companion</b> app on your phone. It connects automatically via UDP beacon on Wi-Fi, or enter PC IP:
                </p>
                <div className="mt-2 p-2 bg-[#181a20] border border-[#262933] rounded-xl flex items-center justify-center gap-2 text-xs font-mono text-indigo-300">
                  <span>IP: {qrData?.server_ip || 'Loading...'}</span>
                  <span>•</span>
                  <span>Port: {qrData?.port || 27890}</span>
                </div>
              </div>
            </div>
          ) : (
            <>
              {/* Instructions Alert */}
              <div className="bg-[#181a20] border border-[#262933] rounded-xl p-3 text-xs text-gray-300 flex flex-col gap-1.5">
                <span className="font-semibold text-indigo-400 flex items-center gap-1.5">
                  <Sparkles size={14} /> Setup Instructions:
                </span>
                {activeTab === 'pair' ? (
                  <ol className="list-decimal list-inside space-y-1 text-[11px] text-gray-400">
                    <li>On your phone, go to <b>Developer Options → Wireless Debugging</b>.</li>
                    <li>Tap <b>Pair device with pairing code</b>.</li>
                    <li>Enter the shown IP, <b>Pairing Port</b>, and <b>6-digit Code</b> below.</li>
                  </ol>
                ) : (
                  <ol className="list-decimal list-inside space-y-1 text-[11px] text-gray-400">
                    <li>Turn on <b>Wireless Debugging</b> on your phone.</li>
                    <li>Look at the IP address and <b>Connect Port</b> on the main screen.</li>
                    <li>Enter them below and click Connect.</li>
                  </ol>
                )}
              </div>

              {/* mDNS Discovered devices badge */}
              {discoveredServices.length > 0 && (
                <div className="flex flex-col gap-1.5">
                  <div className="flex items-center justify-between text-[11px] text-gray-400">
                    <span>Discovered on Wi-Fi (mDNS):</span>
                    <button
                      onClick={handleScanMdns}
                      className="flex items-center gap-1 text-indigo-400 hover:underline"
                    >
                      <RefreshCw size={11} className={scanning ? 'animate-spin' : ''} /> Refresh
                    </button>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {discoveredServices.map((srv, idx) => (
                      <button
                        key={idx}
                        onClick={() => selectDiscovered(srv)}
                        className="px-2.5 py-1.5 bg-[#181a20] hover:bg-indigo-600/20 border border-[#262933] hover:border-indigo-500/40 rounded-lg text-xs text-gray-300 flex items-center gap-1.5 transition cursor-pointer"
                      >
                        <Wifi size={12} className="text-emerald-400" />
                        <span>{srv.host}:{srv.port}</span>
                      </button>
                    ))}
                  </div>
                </div>
              )}

              {/* Form */}
              <form onSubmit={activeTab === 'pair' ? handlePair : handleConnect} className="flex flex-col gap-3">
                <div className="grid grid-cols-3 gap-2.5">
                  <div className="col-span-2 flex flex-col gap-1">
                    <label className="text-[11px] font-medium text-gray-400">Phone IP Address</label>
                    <input
                      type="text"
                      value={host}
                      onChange={(e) => setHost(e.target.value)}
                      placeholder="192.168.1.50"
                      className="bg-[#111317] border border-[#262933] focus:border-indigo-500 rounded-lg px-3 py-2 text-xs text-white outline-hidden font-mono"
                      required
                    />
                  </div>

                  <div className="flex flex-col gap-1">
                    <label className="text-[11px] font-medium text-gray-400">Port</label>
                    <input
                      type="text"
                      value={port}
                      onChange={(e) => setPort(e.target.value)}
                      placeholder={activeTab === 'pair' ? '38492' : '41235'}
                      className="bg-[#111317] border border-[#262933] focus:border-indigo-500 rounded-lg px-3 py-2 text-xs text-white outline-hidden font-mono"
                      required
                    />
                  </div>
                </div>

                {activeTab === 'pair' && (
                  <div className="flex flex-col gap-1">
                    <label className="text-[11px] font-medium text-gray-400">6-Digit Pairing Code</label>
                    <div className="relative">
                      <KeyRound size={15} className="absolute left-3 top-2.5 text-gray-500" />
                      <input
                        type="text"
                        value={code}
                        onChange={(e) => setCode(e.target.value)}
                        placeholder="482910"
                        maxLength={8}
                        className="w-full bg-[#111317] border border-[#262933] focus:border-indigo-500 rounded-lg pl-9 pr-3 py-2 text-xs text-white tracking-widest font-mono outline-hidden"
                        required
                      />
                    </div>
                  </div>
                )}

                {/* Status message */}
                {statusMsg && (
                  <div
                    className={`p-2.5 rounded-lg text-xs flex items-center gap-2 ${
                      statusMsg.type === 'success'
                        ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                        : statusMsg.type === 'error'
                        ? 'bg-rose-500/10 text-rose-400 border border-rose-500/20'
                        : 'bg-indigo-500/10 text-indigo-400 border border-indigo-500/20'
                    }`}
                  >
                    {statusMsg.type === 'success' ? (
                      <CheckCircle2 size={14} className="shrink-0" />
                    ) : (
                      <AlertCircle size={14} className="shrink-0" />
                    )}
                    <span>{statusMsg.text}</span>
                  </div>
                )}

                <button
                  type="submit"
                  disabled={loading}
                  className="mt-2 w-full py-2.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white text-xs font-semibold rounded-xl transition shadow-lg shadow-indigo-600/20 flex items-center justify-center gap-2 cursor-pointer active:scale-95"
                >
                  {loading ? (
                    <RefreshCw size={14} className="animate-spin" />
                  ) : (
                    <ArrowRight size={14} />
                  )}
                  {activeTab === 'pair' ? 'Pair With Device' : 'Connect Device'}
                </button>
              </form>
            </>
          )}
        </div>
      </div>
    </div>
  );
};
