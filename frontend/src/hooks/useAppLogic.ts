import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { listen } from '@tauri-apps/api/event'
import { useSettings } from './useSettings'
import { useConnection } from './useConnection'
import { useRuntime } from './useRuntime'
import { useTheme } from './useTheme'
import { formatErrorLog, toAppError, type AppError } from '../lib/errors'
import {
  INITIAL_SETUP_STEPS,
  updateSetupSteps,
  type SetupProgressPayload,
  type SetupStepItem,
} from '../lib/setupSteps'
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
  const connectionError: AppError | null =
    connection.error ?? (connection.result.kind === 'error' ? connection.result.error : null)

  const [wirelessSetupStatus, setWirelessSetupStatus] = useState<
    'idle' | 'loading' | 'success' | 'error'
  >('idle')
  const [setupSteps, setSetupSteps] = useState<SetupStepItem[]>(INITIAL_SETUP_STEPS)
  const autoStartedRef = useRef(false)

  useEffect(() => {
    if (autoStartedRef.current) return
    if (connection.autoStatus === 'skipped') {
      autoStartedRef.current = true
      return
    }
    if (
      connection.autoStatus !== 'success' ||
      !connection.connected ||
      connection.busy ||
      settings.isSaving ||
      !runtime.ready ||
      runtime.busy ||
      !mode
    ) {
      return
    }
    autoStartedRef.current = true
    if (config?.steamvr_auto_launch && !runtime.running) {
      const interval = config.keep_awake_interval_secs ?? 3
      void runtime.toggle(mode, true, false).then((ok) => {
        if (ok) {
          runtime.addLog(t('log_started_loop', { interval }))
        }
      })
    }
  }, [
    config?.steamvr_auto_launch,
    config?.keep_awake_interval_secs,
    connection.autoStatus,
    connection.connected,
    connection.busy,
    settings.isSaving,
    runtime,
    mode,
    t,
  ])

  useEffect(() => {
    const unlisten = listen<SetupProgressPayload>('setup-progress', (event) => {
      setSetupSteps((prev) => updateSetupSteps(prev, event.payload))
      if (event.payload.state === 'failed') {
        const detailStr = event.payload.detail ? ` (${event.payload.detail})` : ''
        runtime.addLog(`${t(`setup_step_${event.payload.step}`)}: failed${detailStr}`)
      }
    }).catch(() => () => {})

    return () => {
      void unlisten.then((stop) => stop())
    }
  }, [t, runtime.addLog])

  useEffect(() => {
    if (connectionError) {
      runtime.addLog(formatErrorLog(connectionError, t, runtime.debug))
    }
  }, [connectionError, runtime.addLog, runtime.debug, t])

  useEffect(() => {
    if (runtime.error) {
      const appErr = toAppError(runtime.error)
      runtime.addLog(formatErrorLog(appErr, t, runtime.debug))
    }
  }, [runtime.error, runtime.addLog, runtime.debug, t])

  const resetSetup = useCallback(() => {
    setSetupSteps(INITIAL_SETUP_STEPS)
    setWirelessSetupStatus('idle')
    connection.clearError()
  }, [connection.clearError])

  const setupWireless = async () => {
    setSetupSteps(INITIAL_SETUP_STEPS)
    setWirelessSetupStatus('loading')
    runtime.addLog(t('log_wireless_init'))
    const connected = await connection.setup()
    setWirelessSetupStatus(connected ? 'success' : 'error')
    if (connected) {
      runtime.addLog(t('log_wireless_complete'))
    }
    return connected
  }

  const checkDevices = async () => {
    setSetupSteps(INITIAL_SETUP_STEPS)
    runtime.addLog(t('log_checking_devices'))
    const connected = await connection.connect('wired')
    if (connected) {
      runtime.addLog(t('wired_status_success'))
    }
    return connected
  }

  const connectManual = async (ip?: string) => {
    const address = ip ?? window.prompt(t('prompt_enter_ip'), config?.ip_address ?? '')
    if (!address) return false
    runtime.addLog(t('log_connecting_manual', { ip: address }))
    const connected = await connection.connect('wireless', address)
    if (connected) {
      runtime.addLog(t('log_auto_connect_success', { ip: address }))
    }
    return connected
  }

  const toggleKeepAwake = async () => {
    const wasRunning = runtime.running
    const interval = config?.keep_awake_interval_secs ?? 3
    const ok = await runtime.toggle(
      mode,
      connection.connected,
      connection.busy || settings.isSaving,
    )
    if (ok) {
      runtime.addLog(wasRunning ? t('log_stopped_loop') : t('log_started_loop', { interval }))
    }
  }

  const toggleDebugMode = async (enabled: boolean) => {
    await runtime.toggleDebug(enabled)
    runtime.addLog(t(enabled ? 'log_debug_enabled' : 'log_debug_disabled'))
  }

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
    setupSteps,
    resetSetup,
    logs: runtime.logs,
    isRunning: runtime.running,
    isDebug: runtime.debug,
    isConnecting: connection.busy,
    runtimeError: runtime.error ? toAppError(runtime.error) : null,
    deviceIp: config?.ip_address ?? '',
    connectionMode: mode,
    isConnected: connection.connected,
    connectionStatus: status,
    connectionError,
    dimAfterHours: config?.dim_delay_hours ?? 1,
    keepAwakeInterval: config?.keep_awake_interval_secs ?? 3,
    steamVrAutoLaunch: config?.steamvr_auto_launch ?? false,
    changeLanguage: (language: string) => {
      void i18n.changeLanguage(language)
    },
    toggleDebugMode,
    handleModeSelect: (next: ConnectionMode) => connection.selectMode(next),
    setConnectionMode: connection.selectMode,
    checkDevices,
    setupWireless,
    connectManual,
    toggleKeepAwake,
    updateDimDelay: (hours: number) => settings.update({ dim_delay_hours: hours }),
    updateKeepAwakeInterval: (seconds: number) =>
      settings.update({ keep_awake_interval_secs: seconds }),
    updateSteamVrAutoLaunch: (enabled: boolean) =>
      settings.update({ steamvr_auto_launch: enabled }),
    autoConnectStatus: connection.autoStatus,
    autoConnectType: mode,
    autoConnectAttempt: connection.attempt,
    retryAutoConnect: () => connection.connect(mode ?? 'wired', config?.ip_address ?? '', true),
    setAutoConnectStatus: (_value: string) => connection.dismiss(),
    getDeviceModel,
    wirelessSetupStatus,
    wiredSetupStatus: (connection.busy
      ? 'loading'
      : connectionError
        ? 'error'
        : status?.is_usb_connected
          ? 'success'
          : 'idle') as 'loading' | 'error' | 'idle' | 'success',
  }
}
