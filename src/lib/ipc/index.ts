import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
  DeviceInfo,
  ConnectionState,
  ConnectionStatusEvent,
  NotificationItem,
  DeviceTelemetry,
  DiscoveredService,
  DeviceCapabilities,
} from '../../types';

// Helper to safely invoke Tauri commands with mock fallback when opened directly in a web browser
async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const inTauri = typeof window !== 'undefined' && (isTauri() || '__TAURI_INTERNALS__' in window);

  if (inTauri) {
    return await invoke<T>(cmd, args);
  }

  console.warn(`[Mock Browser Mode] Executing '${cmd}' with args:`, args);

  // Mock responses for development inside normal web browser (Chrome/Edge) without Tauri container
  if (cmd === 'check_adb_status') {
    return 'Android Debug Bridge version 1.0.41 (Browser Mock)' as unknown as T;
  }
  if (cmd === 'pair_device') {
    const host = args?.host as string;
    const port = args?.port as number;
    return `Successfully paired to ${host}:${port} [guid=mock-device-guid]` as unknown as T;
  }
  if (cmd === 'connect_device') {
    const host = args?.host as string;
    const port = args?.port as number;
    const mockDevice: DeviceInfo = {
      serial: `${host}:${port}`,
      model: 'Pixel 8 Pro (Simulated)',
      manufacturer: 'Google',
      android_version: '14',
      sdk_version: '34',
      is_wireless: true,
      is_connected: true,
    };
    return mockDevice as unknown as T;
  }
  if (cmd === 'get_telemetry') {
    const mockTelemetry: DeviceTelemetry = {
      battery_level: 85,
      battery_status: 'charging',
      battery_temp_celsius: 31.4,
      storage_free_gb: 64.2,
      storage_total_gb: 128.0,
      storage_used_percent: 50,
      wifi_signal_dbm: -52,
      wifi_ssid: 'Home_Wi-Fi_5G',
    };
    return mockTelemetry as unknown as T;
  }
  if (cmd === 'get_connection_state') {
    return 'disconnected' as unknown as T;
  }
  if (cmd === 'get_active_device') {
    return null as unknown as T;
  }
  if (cmd === 'scan_mdns') {
    return [] as unknown as T;
  }
  if (cmd === 'get_notification_history') {
    return [] as unknown as T;
  }
  if (cmd === 'copy_otp_to_clipboard') {
    if (typeof navigator !== 'undefined' && navigator.clipboard) {
      await navigator.clipboard.writeText(args?.code as string);
    }
    return undefined as unknown as T;
  }

  return undefined as unknown as T;
}

export const tauriApi = {
  // ADB & Device Connection
  checkAdbStatus: (): Promise<string> => safeInvoke('check_adb_status'),
  pairDevice: (host: string, port: number, code: string): Promise<string> =>
    safeInvoke('pair_device', { host, port, code }),
  connectDevice: (host: string, port: number): Promise<DeviceInfo> =>
    safeInvoke('connect_device', { host, port }),
  connectBySerial: (serial: string): Promise<DeviceInfo> =>
    safeInvoke('connect_by_serial', { serial }),
  getSavedDevices: (): Promise<import('../../types').SavedDevice[]> =>
    safeInvoke('get_saved_devices'),
  deleteSavedDevice: (serial: string): Promise<void> =>
    safeInvoke('delete_saved_device', { serial }),
  disconnectDevice: (): Promise<void> => safeInvoke('disconnect_device'),
  getConnectionState: (): Promise<ConnectionState> => safeInvoke('get_connection_state'),
  getActiveDevice: (): Promise<DeviceInfo | null> => safeInvoke('get_active_device'),
  scanMdns: (): Promise<DiscoveredService[]> => safeInvoke('scan_mdns'),

  // Telemetry
  getTelemetry: (serial: string): Promise<DeviceTelemetry> =>
    safeInvoke('get_telemetry', { serial }),

  // Remote Controls
  volumeUp: (serial: string): Promise<void> => safeInvoke('volume_up', { serial }),
  volumeDown: (serial: string): Promise<void> => safeInvoke('volume_down', { serial }),
  mediaPlayPause: (serial: string): Promise<void> => safeInvoke('media_play_pause', { serial }),
  mediaNext: (serial: string): Promise<void> => safeInvoke('media_next', { serial }),
  mediaPrev: (serial: string): Promise<void> => safeInvoke('media_prev', { serial }),
  wakeScreen: (serial: string): Promise<void> => safeInvoke('wake_screen', { serial }),
  lockScreen: (serial: string): Promise<void> => safeInvoke('lock_screen', { serial }),
  checkCapabilities: (serial: string): Promise<DeviceCapabilities> =>
    safeInvoke('check_capabilities', { serial }),

  // Security & Clipboard
  copyOtpToClipboard: (code: string, ttlSecs?: number): Promise<void> =>
    safeInvoke('copy_otp_to_clipboard', { code, ttlSecs }),

  // Companion App Integration
  getCompanionPairingQr: (): Promise<import('../../types').PairingQrData> =>
    safeInvoke('get_companion_pairing_qr'),
  getConnectedCompanion: (): Promise<import('../../types').ConnectedCompanion | null> =>
    safeInvoke('get_connected_companion'),
  sendCompanionReply: (key: string, replyText: string): Promise<void> =>
    safeInvoke('send_companion_reply', { key, replyText }),

  // Notification Storage
  getNotificationHistory: (limit?: number): Promise<NotificationItem[]> =>
    safeInvoke('get_notification_history', { limit }),
  deleteNotification: (id: string): Promise<void> =>
    safeInvoke('delete_notification', { id }),
  clearNotificationHistory: (): Promise<void> => safeInvoke('clear_notification_history'),

  // Event Listeners
  onConnectionStatusChanged: async (callback: (event: ConnectionStatusEvent) => void): Promise<UnlistenFn> => {
    try {
      if (typeof window !== 'undefined' && (isTauri() || '__TAURI_INTERNALS__' in window)) {
        return await listen<ConnectionStatusEvent>('connection-status-changed', (e) => callback(e.payload));
      }
    } catch (e) {
      console.warn('Browser mode: connection listener mock active');
    }
    return () => {};
  },
  onNotificationReceived: async (callback: (item: NotificationItem) => void): Promise<UnlistenFn> => {
    try {
      if (typeof window !== 'undefined' && (isTauri() || '__TAURI_INTERNALS__' in window)) {
        return await listen<NotificationItem>('notification-received', (e) => callback(e.payload));
      }
    } catch (e) {
      console.warn('Browser mode: notification listener mock active');
    }
    return () => {};
  },
  onTelemetryUpdated: async (callback: (item: DeviceTelemetry) => void): Promise<UnlistenFn> => {
    try {
      if (typeof window !== 'undefined' && (isTauri() || '__TAURI_INTERNALS__' in window)) {
        return await listen<DeviceTelemetry>('telemetry-updated', (e) => callback(e.payload));
      }
    } catch (e) {
      console.warn('Browser mode: telemetry listener mock active');
    }
    return () => {};
  },
};
