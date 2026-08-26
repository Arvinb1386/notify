import { create } from 'zustand';
import { DeviceInfo, ConnectionState, DeviceTelemetry, DeviceCapabilities, SavedDevice } from '../types';
import { tauriApi } from '../lib/ipc';

interface DeviceStoreState {
  activeDevice: DeviceInfo | null;
  connectionState: ConnectionState;
  statusMessage: string | null;
  telemetry: DeviceTelemetry | null;
  capabilities: DeviceCapabilities | null;
  savedDevices: SavedDevice[];
  isPairingModalOpen: boolean;

  setConnectionState: (state: ConnectionState, msg?: string | null) => void;
  setActiveDevice: (device: DeviceInfo | null) => void;
  setTelemetry: (telemetry: DeviceTelemetry | null) => void;
  setCapabilities: (cap: DeviceCapabilities | null) => void;
  setPairingModalOpen: (open: boolean) => void;

  initConnectionListeners: () => Promise<() => void>;
  fetchTelemetry: () => Promise<void>;
  loadSavedDevices: () => Promise<void>;
  connectSavedDevice: (serial: string) => Promise<void>;
  deleteSavedDevice: (serial: string) => Promise<void>;
  disconnect: () => Promise<void>;
}

export const useDeviceStore = create<DeviceStoreState>((set, get) => ({
  activeDevice: null,
  connectionState: 'disconnected',
  statusMessage: null,
  telemetry: null,
  capabilities: null,
  savedDevices: [],
  isPairingModalOpen: false,

  setConnectionState: (connectionState, statusMessage) =>
    set({ connectionState, statusMessage: statusMessage ?? null }),
  setActiveDevice: (activeDevice) => set({ activeDevice }),
  setTelemetry: (telemetry) => set({ telemetry }),
  setCapabilities: (capabilities) => set({ capabilities }),
  setPairingModalOpen: (isPairingModalOpen) => set({ isPairingModalOpen }),

  loadSavedDevices: async () => {
    try {
      const list = await tauriApi.getSavedDevices();
      set({ savedDevices: list });
    } catch (e) {
      console.debug('Failed to load saved devices:', e);
    }
  },

  connectSavedDevice: async (serial: string) => {
    set({ connectionState: 'connecting', statusMessage: `Connecting to ${serial}...` });
    try {
      const dev = await tauriApi.connectBySerial(serial);
      set({ activeDevice: dev, connectionState: 'connected', statusMessage: null });
      get().loadSavedDevices();
      get().fetchTelemetry();
    } catch (e: any) {
      set({ connectionState: 'disconnected', statusMessage: e?.message || 'Connection failed' });
    }
  },

  deleteSavedDevice: async (serial: string) => {
    try {
      await tauriApi.deleteSavedDevice(serial);
      get().loadSavedDevices();
    } catch (e) {
      console.error('Delete saved device error:', e);
    }
  },

  initConnectionListeners: async () => {
    get().loadSavedDevices();
    // Initial fetch
    try {
      const state = await tauriApi.getConnectionState();
      const dev = await tauriApi.getActiveDevice();
      set({ connectionState: state, activeDevice: dev });
      if (dev) {
        get().fetchTelemetry();
      }
    } catch (e) {
      console.warn('Initial connection fetch error:', e);
    }

    const unlistenConn = await tauriApi.onConnectionStatusChanged((event) => {
      set({
        connectionState: event.state,
        activeDevice: event.device || null,
        statusMessage: event.message || null,
      });

      if (event.state === 'connected' && event.device) {
        get().loadSavedDevices();
        get().fetchTelemetry();
      } else if (event.state === 'disconnected') {
        set({ telemetry: null, capabilities: null });
      }
    });

    const unlistenTelemetry = await tauriApi.onTelemetryUpdated((telemetry) => {
      set({ telemetry });
    });

    return () => {
      unlistenConn();
      unlistenTelemetry();
    };
  },

  fetchTelemetry: async () => {
    const { activeDevice } = get();
    if (!activeDevice) return;
    try {
      const data = await tauriApi.getTelemetry(activeDevice.serial);
      set({ telemetry: data });
    } catch (e) {
      console.debug('Telemetry fetch failed:', e);
    }
  },

  disconnect: async () => {
    try {
      await tauriApi.disconnectDevice();
      set({
        activeDevice: null,
        connectionState: 'disconnected',
        telemetry: null,
        statusMessage: 'Disconnected',
      });
    } catch (e) {
      console.error('Disconnect error:', e);
    }
  },
}));
