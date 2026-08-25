import { invoke } from '@tauri-apps/api/core';
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

export const tauriApi = {
  // ADB & Device Connection
  checkAdbStatus: (): Promise<string> => invoke('check_adb_status'),
  pairDevice: (host: string, port: number, code: string): Promise<string> =>
    invoke('pair_device', { host, port, code }),
  connectDevice: (host: string, port: number): Promise<DeviceInfo> =>
    invoke('connect_device', { host, port }),
  disconnectDevice: (): Promise<void> => invoke('disconnect_device'),
  getConnectionState: (): Promise<ConnectionState> => invoke('get_connection_state'),
  getActiveDevice: (): Promise<DeviceInfo | null> => invoke('get_active_device'),
  scanMdns: (): Promise<DiscoveredService[]> => invoke('scan_mdns'),

  // Telemetry
  getTelemetry: (serial: string): Promise<DeviceTelemetry> =>
    invoke('get_telemetry', { serial }),

  // Remote Controls
  volumeUp: (serial: string): Promise<void> => invoke('volume_up', { serial }),
  volumeDown: (serial: string): Promise<void> => invoke('volume_down', { serial }),
  mediaPlayPause: (serial: string): Promise<void> => invoke('media_play_pause', { serial }),
  mediaNext: (serial: string): Promise<void> => invoke('media_next', { serial }),
  mediaPrev: (serial: string): Promise<void> => invoke('media_prev', { serial }),
  wakeScreen: (serial: string): Promise<void> => invoke('wake_screen', { serial }),
  lockScreen: (serial: string): Promise<void> => invoke('lock_screen', { serial }),
  checkCapabilities: (serial: string): Promise<DeviceCapabilities> =>
    invoke('check_capabilities', { serial }),

  // Security & Clipboard
  copyOtpToClipboard: (code: string, ttlSecs?: number): Promise<void> =>
    invoke('copy_otp_to_clipboard', { code, ttlSecs }),

  // Notification Storage
  getNotificationHistory: (limit?: number): Promise<NotificationItem[]> =>
    invoke('get_notification_history', { limit }),
  clearNotificationHistory: (): Promise<void> => invoke('clear_notification_history'),

  // Event Listeners
  onConnectionStatusChanged: (callback: (event: ConnectionStatusEvent) => void): Promise<UnlistenFn> =>
    listen<ConnectionStatusEvent>('connection-status-changed', (e) => callback(e.payload)),
  onNotificationReceived: (callback: (item: NotificationItem) => void): Promise<UnlistenFn> =>
    listen<NotificationItem>('notification-received', (e) => callback(e.payload)),
};
