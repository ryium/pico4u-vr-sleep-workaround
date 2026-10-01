import { useTranslation } from 'react-i18next'
import { useSettings } from './useSettings'
import { useConnection } from './useConnection'
import { useRuntime } from './useRuntime'
import { useTheme } from './useTheme'
import type { ConnectionMode } from '../types'

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
  const mode = settings.config?.last_connection_mode ?? null
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
    deviceIp: settings.config?.ip_address ?? '',
    connectionMode: mode,
    isConnected: connection.connected,
    changeLanguage: (language: string) => {
      void i18n.changeLanguage(language)
    },
    toggleDebugMode: runtime.toggleDebug,
    handleModeSelect: (next: ConnectionMode) => connection.selectMode(next),
    setupWireless: connection.setup,
    connectManual: (ip: string) => connection.connect('wireless', ip),
    toggleKeepAwake: () =>
      runtime.toggle(mode, connection.connected, connection.busy || settings.isSaving),
  }
}
