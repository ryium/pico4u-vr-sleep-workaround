export type ConnectionMode = 'wired' | 'wireless'
export interface AppConfig {
  dim_delay_hours: number
  ip_address: string
  keep_awake_interval_secs: number
  last_connection_mode: ConnectionMode | null
}
export interface ConnectionStatus {
  is_usb_connected: boolean
  is_wifi_ip_available: boolean
  wifi_ip: string | null
  is_adb_tcp_connected: boolean
}
export type ConnectionResult =
  | { kind: 'loading' }
  | { kind: 'success'; status: ConnectionStatus }
  | { kind: 'error'; error: string }
export function isConnected(result: ConnectionResult, mode: ConnectionMode | null) {
  return (
    result.kind === 'success' &&
    (mode === 'wired'
      ? result.status.is_usb_connected
      : mode === 'wireless' && result.status.is_adb_tcp_connected)
  )
}
