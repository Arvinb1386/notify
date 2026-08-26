export interface DeviceInfo {
  serial: string;
  model: string;
  manufacturer: string;
  android_version: string;
  sdk_version: string;
  is_wireless: boolean;
  is_connected: boolean;
}

export type ConnectionState = 'disconnected' | 'discovering' | 'connecting' | 'connected' | 'degraded';

export interface ConnectionStatusEvent {
  state: ConnectionState;
  device?: DeviceInfo | null;
  message?: string | null;
}

export interface NotificationItem {
  id: string;
  package_name: string;
  app_name?: string | null;
  title?: string | null;
  body?: string | null;
  subtext?: string | null;
  channel_id?: string | null;
  post_time: number;
  is_otp: boolean;
  otp_code?: string | null;
  status: 'posted' | 'updated' | 'removed';
  fingerprint: string;
}

export type BatteryStatus = 'charging' | 'discharging' | 'full' | 'notcharging' | 'unknown';

export interface DeviceTelemetry {
  battery_level: number;
  battery_status: BatteryStatus;
  battery_temp_celsius: number;
  storage_free_gb: number;
  storage_total_gb: number;
  storage_used_percent: number;
  wifi_signal_dbm?: number | null;
  wifi_ssid?: string | null;
}

export interface DiscoveredService {
  instance_name: string;
  service_type: string;
  host: string;
  port: number;
}

export interface SavedDevice {
  serial: string;
  model: string;
  manufacturer: string;
  android_version: string;
  last_connected: number;
}

export interface DeviceCapabilities {
  supports_volume: boolean;
  supports_media: boolean;
  supports_wake: boolean;
  supports_lock: boolean;
}
