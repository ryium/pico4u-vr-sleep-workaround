import { useCallback, useEffect, useRef, useState } from 'react'
import { api } from '../lib/api'
import { poll, retry } from '../lib/async'
import type { AppConfig, ConnectionMode, ConnectionResult } from '../types'
import { isConnected } from '../types'

export function useConnection(
  config: AppConfig | null,
  update: (patch: Partial<AppConfig>) => Promise<AppConfig>,
  running: boolean,
) {
  const [result, setResult] = useState<ConnectionResult>({ kind: 'loading' })
  const [busy, setBusy] = useState(false)
  const [autoStatus, setAutoStatus] = useState<
    'idle' | 'connecting' | 'success' | 'failed' | 'skipped'
  >('idle')
  const [attempt, setAttempt] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const lifecycle = useRef<AbortController | null>(null)
  const lock = useRef(false)
  const initial = useRef(false)
  const revision = useRef(0)
  const mode = config?.last_connection_mode ?? null
  const ip = config?.ip_address ?? ''
  useEffect(() => {
    const controller = new AbortController()
    lifecycle.current = controller
    return () => {
      controller.abort()
      initial.current = false
      lock.current = false
    }
  }, [])
  useEffect(() => {
    const controller = new AbortController()
    setResult({ kind: 'loading' })
    if (config)
      poll(
        async () => {
          const generation = revision.current
          const status = await api.status(mode, ip, controller.signal)
          return { generation, status }
        },
        ({ generation, status }) => {
          if (generation === revision.current && !lock.current) {
            setResult({ kind: 'success', status })
            if (isConnected({ kind: 'success', status }, mode)) setError(null)
          }
        },
        (e) => {
          if (!lock.current) setResult({ kind: 'error', error: String(e) })
        },
        controller.signal,
      )
    return () => controller.abort()
  }, [config !== null, mode, ip])

  const perform = useCallback(
    async (task: (signal: AbortSignal) => Promise<AppConfig>, automatic = false) => {
      const controller = lifecycle.current
      if (!controller || controller.signal.aborted || lock.current || running || !config)
        return false
      const signal = controller.signal
      lock.current = true
      revision.current++
      setBusy(true)
      setError(null)
      setResult({ kind: 'loading' })
      if (automatic) setAutoStatus('connecting')
      try {
        const next = await task(signal)
        if (signal.aborted) return false
        const status = await api.status(next.last_connection_mode, next.ip_address, signal)
        if (signal.aborted) return false
        setResult({ kind: 'success', status })
        const connected = isConnected({ kind: 'success', status }, next.last_connection_mode)
        if (automatic) setAutoStatus(connected ? 'success' : 'failed')
        return connected
      } catch (e) {
        if (!signal.aborted) {
          setError(String(e))
          setResult({ kind: 'error', error: String(e) })
          if (automatic) setAutoStatus('failed')
        }
        return false
      } finally {
        if (!signal.aborted) {
          lock.current = false
          setBusy(false)
        }
      }
    },
    [config, running],
  )

  const connect = useCallback(
    (targetMode: ConnectionMode = mode ?? 'wired', targetIp = ip) =>
      perform(async (signal) => {
        // Save the chosen target first so polling and the next start agree.
        const next = await update({ last_connection_mode: targetMode, ip_address: targetIp.trim() })
        await retry(() => api.connect(targetMode, next.ip_address, signal), signal, setAttempt)
        return next
      }, true),
    [perform, mode, ip, update],
  )

  useEffect(() => {
    if (!config || initial.current) return
    initial.current = true
    if (mode && (mode === 'wired' || ip)) void connect(mode, ip)
    else setAutoStatus('skipped')
  }, [config, connect, mode, ip])

  const dismiss = useCallback(() => setAutoStatus('skipped'), [])

  const selectMode = (nextMode: ConnectionMode | null) =>
    perform(async () => {
      setAutoStatus('skipped')
      return update({ last_connection_mode: nextMode })
    })
  const setup = () =>
    perform(async (signal) => {
      const address = await api.setup(signal)
      return update({ last_connection_mode: 'wireless', ip_address: address })
    })
  return {
    result,
    busy,
    error,
    autoStatus,
    attempt,
    connect,
    selectMode,
    setup,
    dismiss,
    connected: isConnected(result, mode),
  }
}
