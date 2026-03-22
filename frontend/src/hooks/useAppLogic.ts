// d:\Repository\pico4u-sleep-workaround\frontend\src\hooks\useAppLogic.ts
import { useState, useEffect, useCallback } from 'react'
import type { ChangeEvent } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useTranslation } from 'react-i18next'
import { useTheme } from './useTheme'

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

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
  const [dimAfterHours, setDimAfterHours] = useState<number>(0)
  const { theme, setTheme } = useTheme()

  // Load config on mount
  useEffect(() => {
    invoke<{ dim_delay_hours: number; ip_address: string }>('get_config')
      .then((config) => {
        setDimAfterHours(config.dim_delay_hours)
        if (config.ip_address) {
          setDeviceIp(config.ip_address)
        }
      })
      .catch((e) => console.error('Failed to load config:', e))
  }, [])

  const updateConfig = useCallback(
    async (newConfig: { dim_delay_hours?: number; ip_address?: string }) => {
      try {
        const configToSave = {
          dim_delay_hours: newConfig.dim_delay_hours ?? dimAfterHours,
          ip_address: newConfig.ip_address ?? deviceIp,
        }
        await invoke('save_config_cmd', { config: configToSave })
      } catch (e) {
        console.error('Failed to save config:', e)
      }
    },
    [dimAfterHours, deviceIp],
  )

  const updateDimDelay = useCallback(
    async (hours: number) => {
      const val = Math.max(0, hours)
      setDimAfterHours(val)
      await updateConfig({ dim_delay_hours: val })
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

  const addLog = useCallback((msg: string) => {
    if (!msg || typeof msg !== 'string') return
    const trimmed = msg.trim()
    if (trimmed.length === 0) return

    setLogs((prev) => [`[${new Date().toLocaleTimeString()}] ${trimmed}`, ...prev])
  }, [])

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
      addLog(t('log_checking_devices'))
      const res = await invoke<string>('check_connection')

      if (res && res.trim()) {
        addLog(res)
      }

      const connected = res.includes('device') && !res.trim().endsWith('List of devices attached')
      setIsConnected(connected)
    } catch (e) {
      addLog(t('error_prefix', { error: e }))
      setIsConnected(false)
    }
  }, [t, addLog])

  const toggleDebugMode = useCallback(
    async (e: ChangeEvent<HTMLInputElement>) => {
      const enabled = e.target.checked
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

      if (mode === 'wired') {
        try {
          addLog(t('log_usb_mode'))
          await invoke('kill_adb')
          await invoke('set_usb_mode')
          addLog(t('log_usb_mode_success'))
          checkDevices()
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
      checkDevices()
    } catch (e) {
      addLog(t('log_wireless_error', { error: e }))
      addLog(t('log_wireless_note'))
      setWirelessSetupStatus('error')
      updateDeviceIp('')
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
        checkDevices()
      } catch (e) {
        addLog(t('error_prefix', { error: e }))
      }
    }
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
        addLog(t('log_started_loop'))
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
    connectManual,
    toggleKeepAwake,
    getDeviceModel,
    dimAfterHours,
    updateDimDelay,
    theme,
    setTheme,
  }
}
