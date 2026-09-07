import { useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { useSettings } from './useSettings'
import { useConnection } from './useConnection'
import { useRuntime } from './useRuntime'
import { useTheme } from './useTheme'
import type { ConnectionMode } from '../types'
export type { ConnectionStatus } from '../types'

export function useAppLogic() {
  const { t, i18n } = useTranslation()
  const settings = useSettings()
  const runtime = useRuntime()
  const connection = useConnection(
    runtime.ready ? settings.config : null,
    settings.update,
    runtime.running || runtime.busy,
  )
  const { theme, setTheme } = useTheme()
  const config = settings.config
  const mode = config?.last_connection_mode ?? null
  const status = connection.result.kind === 'success' ? connection.result.status : null
  const getDeviceModel = useCallback(
    async () => (status?.is_usb_connected ? 'A9210' : null),
    [status?.is_usb_connected],
  )
  return {
    t,
    i18n,
    theme,
    setTheme,
    settings,
    runtime,
    connection,
    logs: runtime.logs,
    isRunning: runtime.running,
    isDebug: runtime.debug,
    deviceIp: config?.ip_address ?? '',
    connectionMode: mode,
    isConnected: connection.connected,
    connectionStatus: status,
    dimAfterHours: config?.dim_delay_hours ?? 1,
    keepAwakeInterval: config?.keep_awake_interval_secs ?? 3,
    changeLanguage: (language: string) => {
      void i18n.changeLanguage(language)
    },
    toggleDebugMode: runtime.toggleDebug,
    handleModeSelect: (next: ConnectionMode) => connection.selectMode(next),
    setConnectionMode: connection.selectMode,
    checkDevices: () => connection.connect('wired'),
    setupWireless: connection.setup,
    connectManual: async (ip?: string) => {
      const address = ip ?? window.prompt(t('prompt_enter_ip'), config?.ip_address ?? '')
      return address ? connection.connect('wireless', address) : false
    },
    toggleKeepAwake: () =>
      runtime.toggle(mode, connection.connected, connection.busy || settings.isSaving),
    updateDimDelay: (hours: number) => settings.update({ dim_delay_hours: hours }),
    updateKeepAwakeInterval: (seconds: number) =>
      settings.update({ keep_awake_interval_secs: seconds }),
    autoConnectStatus: connection.autoStatus,
    autoConnectType: mode,
    autoConnectAttempt: connection.attempt,
    retryAutoConnect: () => connection.connect(),
    // Compatibility with the old view until the dashboard replaces it.
    setAutoConnectStatus: (_value: string) => connection.dismiss(),
    getDeviceModel,
    wirelessSetupStatus: (connection.busy ? 'loading' : connection.error ? 'error' : 'idle') as
      | 'loading'
      | 'error'
      | 'idle'
      | 'success',
    wiredSetupStatus: (connection.busy ? 'loading' : connection.error ? 'error' : 'idle') as
      | 'loading'
      | 'error'
      | 'idle'
      | 'success',
  }
}
