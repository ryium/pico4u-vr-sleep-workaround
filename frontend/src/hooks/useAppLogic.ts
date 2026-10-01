// d:\Repository\pico4u-sleep-workaround\frontend\src\hooks\useAppLogic.ts
import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useTranslation } from 'react-i18next'
import { useTheme } from './useTheme'

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

const invokeWithTimeout = async <T>(cmd: string, args: any, timeoutMs: number): Promise<T> => {
  return Promise.race([
    invoke<T>(cmd, args),
    new Promise<T>((_, reject) =>
      setTimeout(
        () => reject(new Error(`Command ${cmd} timed out after ${timeoutMs}ms`)),
        timeoutMs,
      ),
    ),
  ])
}

export function useAppLogic() {
  const { t, i18n } = useTranslation()
  const [logs, setLogs] = useState<string[]>([])
  const [isRunning, setIsRunning] = useState(false)
  const [deviceIp, setDeviceIp] = useState('')
  const [isDebug, setIsDebug] = useState(false)
  const [connectionMode, setConnectionMode] = useState<'wired' | 'wireless' | null>(null)
  const [isConnected, setIsConnected] = useState(false)
  const [wirelessSetupStatus, setWirelessSetupStatus] = useState<
    'idle' | 'loading' | 'success' | 'error'
  >('idle')
  const [wiredSetupStatus, setWiredSetupStatus] = useState<
    'idle' | 'loading' | 'success' | 'error'
  >('idle')
  const [dimAfterHours, setDimAfterHours] = useState<number>(0)
  const [keepAwakeInterval, setKeepAwakeInterval] = useState<number>(3)
  const { theme, setTheme } = useTheme()

  // Auto-connect state: 'idle' = not tried yet, 'connecting' = in progress,
  // 'success' = connected, 'failed' = fall back to manual, 'skipped' = no saved IP
  const [autoConnectStatus, setAutoConnectStatus] = useState<
    'idle' | 'connecting' | 'success' | 'failed' | 'skipped'
  >('idle')
  const [autoConnectType, setAutoConnectType] = useState<'wired' | 'wireless' | null>(null)
  const [autoConnectAttempt, setAutoConnectAttempt] = useState(1)

  const initialized = useRef(false)

  const addLog = useCallback((msg: string) => {
    if (!msg || typeof msg !== 'string') return
    const trimmed = msg.trim()
    if (trimmed.length === 0) return

    setLogs((prev) => [`[${new Date().toLocaleTimeString()}] ${trimmed}`, ...prev])
  }, [])

  // Initialize: load config and try auto-connect
  useEffect(() => {
    if (initialized.current) return
    initialized.current = true

    let cancelled = false

    const initApp = async () => {
      try {
        const config = await invoke<{
          dim_delay_hours: number
          ip_address: string
          keep_awake_interval_secs: number
          last_connection_mode: 'wired' | 'wireless' | null
        }>('get_config')

        if (cancelled) return

        setDimAfterHours(config.dim_delay_hours)
        setKeepAwakeInterval(config.keep_awake_interval_secs)

        const savedIp = config.ip_address
        if (savedIp) {
          setDeviceIp(savedIp)
        }

        const lastMode = config.last_connection_mode

        // If no previous mode is saved, we don't auto-connect
        if (!lastMode) {
          setAutoConnectStatus('skipped')
          return
        }

        setAutoConnectStatus('connecting')
        setAutoConnectType(lastMode)

        // Retry up to 3 times for either mode
        const maxRetries = 3
        for (let attempt = 1; attempt <= maxRetries; attempt++) {
          setAutoConnectAttempt(attempt)
          try {
            // Use 10-second timeout for the whole operation (backend has 5s per command)
            const res = await invokeWithTimeout<string>(
              'try_auto_connect',
              { mode: lastMode, ip: savedIp || '' },
              10000,
            )

            if (cancelled) return

            if (res === 'wired' || res === 'wireless') {
              setConnectionMode(res as 'wired' | 'wireless')
              setIsConnected(true)
              setAutoConnectStatus('success')
              if (res === 'wired') {
                addLog(t('wired_status_success'))
              } else {
                addLog(t('log_auto_connect_success', { ip: savedIp }))
              }
              return
            }
          } catch (e) {
            console.warn(`${lastMode} auto-connect attempt ${attempt} failed:`, e)
          }

          if (cancelled) return
          if (attempt < maxRetries) {
            await sleep(2000)
          }
        }

        // All attempts failed
        setAutoConnectStatus('failed')
        if (lastMode === 'wired') {
          addLog(t('wired_status_error'))
        } else {
          addLog(t('log_auto_connect_failed', { ip: savedIp }))
        }
      } catch (e) {
        console.error('Failed to init app:', e)
        if (!cancelled) setAutoConnectStatus('skipped')
      }
    }

    initApp()
    return () => {
      cancelled = true
      initialized.current = false
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const updateConfig = useCallback(
    async (newConfig: {
      dim_delay_hours?: number
      ip_address?: string
      keep_awake_interval_secs?: number
      last_connection_mode?: 'wired' | 'wireless' | null
    }) => {
      try {
        const configToSave = {
          dim_delay_hours: newConfig.dim_delay_hours ?? dimAfterHours,
          ip_address: newConfig.ip_address ?? deviceIp,
          keep_awake_interval_secs: newConfig.keep_awake_interval_secs ?? keepAwakeInterval,
          last_connection_mode: newConfig.last_connection_mode ?? connectionMode,
        }
        await invoke('save_config_cmd', { config: configToSave })
      } catch (e) {
        console.error('Failed to save config:', e)
      }
    },
    [dimAfterHours, deviceIp, keepAwakeInterval, connectionMode],
  )

  const updateDimDelay = useCallback(
    async (hours: number) => {
      const val = Math.max(0, hours)
      setDimAfterHours(val)
      await updateConfig({ dim_delay_hours: val })
    },
    [updateConfig],
  )

  const updateKeepAwakeInterval = useCallback(
    async (secs: number) => {
      const val = Math.max(1, secs)
      setKeepAwakeInterval(val)
      await updateConfig({ keep_awake_interval_secs: val })
    },
    [updateConfig],
  )

  const updateDeviceIp = useCallback(
    async (ip: string) => {
      setDeviceIp(ip)
      await updateConfig({ ip_address: ip })
    },
    [updateConfig],
  )

  // Setup debug log listener
  useEffect(() => {
    const unlistenPromise = listen<string>('debug-log', (event) => {
      if (event.payload) {
        addLog(`[DEBUG] ${event.payload}`)
      }
    }).catch((e) => {
      console.warn('Failed to setup debug listener (ignore if in browser)', e)
      return () => {}
    })

    return () => {
      unlistenPromise.then((unlistenFn) => {
        if (unlistenFn) unlistenFn()
      })
    }
  }, [addLog])

  const changeLanguage = useCallback(
    (lang: string) => {
      i18n.changeLanguage(lang)
    },
    [i18n],
  )

  const checkDevices = useCallback(async () => {
    try {
      setWiredSetupStatus('loading')
      addLog(t('log_checking_devices'))
      const res = await invoke<string>('check_connection')

      if (res && res.trim()) {
        addLog(res)
      }

      const connected = res.includes('device') && !res.trim().endsWith('List of devices attached')
      setIsConnected(connected)
      setWiredSetupStatus(connected ? 'success' : 'error')
      return connected
    } catch (e) {
      addLog(t('error_prefix', { error: e }))
      setIsConnected(false)
      setWiredSetupStatus('error')
      return false
    }
  }, [t, addLog])

  const toggleDebugMode = useCallback(
    async (enabled: boolean) => {
      setIsDebug(enabled)
      try {
        await invoke('set_debug_mode', { enabled })
        addLog(enabled ? t('log_debug_enabled') : t('log_debug_disabled'))
      } catch (err) {
        addLog(t('error_prefix', { error: err }))
      }
    },
    [t, addLog],
  )

  const handleModeSelect = useCallback(
    async (mode: 'wired' | 'wireless') => {
      setConnectionMode(mode)
      setIsConnected(false)
      await updateConfig({ last_connection_mode: mode })

      if (mode === 'wired') {
        try {
          addLog(t('log_usb_mode'))
          // Disconnect any WiFi-connected devices first to avoid "more than one device" errors
          await invoke('disconnect_all_wireless').catch(() => {})

          // Check if device is already responding via USB before restarting ADB
          const preCheck = await invoke<string>('check_connection').catch(() => '')
          const alreadyConnected =
            preCheck.includes('device') && !preCheck.trim().endsWith('List of devices attached')

          if (!alreadyConnected) {
            await invoke('kill_adb')
            await invoke('set_usb_mode')
            // Wait for ADB server to restart and re-detect device
            await sleep(2000)
          }
          addLog(t('log_usb_mode_success'))
          await checkDevices()
        } catch (e) {
          addLog(t('error_prefix', { error: e }))
        }
      }
    },
    [t, addLog, checkDevices],
  )

  const setupWireless = useCallback(async () => {
    try {
      setWirelessSetupStatus('loading')
      addLog(t('log_wireless_init'))
      addLog(t('log_enable_tcpip'))
      await invoke('enable_tcpip')
      addLog(t('log_tcpip_enabled'))

      addLog(t('log_wait_restart'))
      await sleep(5000)

      addLog(t('log_getting_ip'))
      let ip = ''
      let retries = 3

      while (retries > 0) {
        try {
          ip = await invoke<string>('get_device_ip')
          if (ip && ip.trim()) break
          throw new Error('Empty IP received')
        } catch (e) {
          retries--
          if (retries === 0) throw e
          addLog(t('log_retry_ip', { remaining: retries }))
          await sleep(2000)
        }
      }

      updateDeviceIp(ip)
      addLog(t('log_ip_found', { ip }))

      addLog(t('log_connecting', { ip }))
      const res = await invoke<string>('connect_device', { ip })
      if (res && res.trim()) addLog(res)

      addLog(t('log_wireless_complete'))
      setWirelessSetupStatus('success')
      await checkDevices()
      return true
    } catch (e) {
      addLog(t('log_wireless_error', { error: e }))
      addLog(t('log_wireless_note'))
      setWirelessSetupStatus('error')
      updateDeviceIp('')
      return false
    }
  }, [t, addLog, checkDevices, updateDeviceIp])

  const connectManual = useCallback(async () => {
    const ip = prompt(t('prompt_enter_ip'), deviceIp)
    if (ip) {
      try {
        addLog(t('log_connecting_manual', { ip }))
        const res = await invoke<string>('connect_device', { ip })
        if (res && res.trim()) addLog(res)
        updateDeviceIp(ip)
        await checkDevices()
        return true
      } catch (e) {
        addLog(t('error_prefix', { error: e }))
        return false
      }
    }
    return false
  }, [t, addLog, checkDevices, updateDeviceIp, deviceIp])

  const toggleKeepAwake = useCallback(async () => {
    try {
      if (isRunning) {
        await invoke('stop_keep_awake')
        setIsRunning(false)
        addLog(t('log_stopped_loop'))
      } else {
        await invoke('start_keep_awake', { mode: connectionMode })
        setIsRunning(true)
        addLog(t('log_started_loop', { interval: keepAwakeInterval }))
      }
    } catch (e) {
      addLog(t('error_prefix', { error: e }))
    }
  }, [isRunning, connectionMode, t, addLog])

  const getDeviceModel = useCallback(async () => {
    try {
      return await invoke<string>('get_device_model')
    } catch (e) {
      console.error('Failed to get device model:', e)
      return null
    }
  }, [])

  const retryAutoConnect = useCallback(async () => {
    setAutoConnectStatus('connecting')

    try {
      if (autoConnectType === 'wired') {
        const res = await invokeWithTimeout<string>(
          'try_auto_connect',
          { mode: 'wired', ip: '' },
          5000,
        )
        if (res === 'wired') {
          setAutoConnectStatus('success')
          setConnectionMode('wired')
          setIsConnected(true)
          addLog(t('wired_status_success'))
        } else {
          throw new Error('Wired detection failed')
        }
      } else {
        if (!deviceIp || deviceIp.trim() === '') {
          setAutoConnectStatus('skipped')
          return
        }
        const res = await invokeWithTimeout<string>(
          'try_auto_connect',
          { mode: 'wireless', ip: deviceIp },
          8000,
        )
        if (res === 'wireless') {
          setAutoConnectStatus('success')
          setConnectionMode('wireless')
          setIsConnected(true)
          addLog(t('log_auto_connect_success', { ip: deviceIp }))
        } else {
          throw new Error('Wireless connection failed')
        }
      }
    } catch (e) {
      console.error('Retry failed:', e)
      setAutoConnectStatus('failed')
      if (autoConnectType === 'wired') {
        addLog(t('wired_status_error'))
      } else {
        addLog(t('log_auto_connect_failed', { ip: deviceIp }))
      }
    }
  }, [deviceIp, t, addLog, autoConnectType])

  return {
    t,
    i18n,
    logs,
    isRunning,
    deviceIp,
    isDebug,
    connectionMode,
    isConnected,
    setConnectionMode,
    changeLanguage,
    toggleDebugMode,
    handleModeSelect,
    checkDevices,
    setupWireless,
    wirelessSetupStatus,
    wiredSetupStatus,
    connectManual,
    toggleKeepAwake,
    getDeviceModel,
    dimAfterHours,
    updateDimDelay,
    keepAwakeInterval,
    updateKeepAwakeInterval,
    theme,
    setTheme,
    autoConnectStatus,
    setAutoConnectStatus,
    autoConnectType,
    autoConnectAttempt,
    retryAutoConnect,
  }
}
