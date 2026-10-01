import { invoke } from '@tauri-apps/api/core'
import { serialQueue, withTimeout } from './async'
import type { AppConfig, ConnectionMode, ConnectionStatus } from '../types'

const enqueue = serialQueue()
function call<T>(command: string, args: Record<string, unknown>, signal: AbortSignal) {
  return withTimeout(
    enqueue(() => invoke<T>(command, args), signal),
    30000,
    signal,
  )
}
export const api = {
  config: (signal: AbortSignal) => call<AppConfig>('get_config', {}, signal),
  save: (config: AppConfig, signal: AbortSignal) =>
    call<void>('save_config_cmd', { config }, signal),
  status: (mode: ConnectionMode | null, ip: string, signal: AbortSignal) =>
    call<ConnectionStatus>('check_connection_status', { mode, ip }, signal),
  connect: (mode: ConnectionMode, ip: string, signal: AbortSignal) =>
    call<ConnectionMode>('try_auto_connect', { mode, ip }, signal),
  setup: (signal: AbortSignal) => call<string>('setup_wireless', {}, signal),
  running: (signal: AbortSignal) => call<boolean>('get_running', {}, signal),
  start: (mode: ConnectionMode, signal: AbortSignal) =>
    call<void>('start_keep_awake', { mode }, signal),
  stop: (signal: AbortSignal) => call<void>('stop_keep_awake', {}, signal),
  debug: (enabled: boolean, signal: AbortSignal) =>
    call<void>('set_debug_mode', { enabled }, signal),
}
