import { useEffect, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { api } from '../lib/api'
import type { ConnectionMode } from '../types'

export function useRuntime() {
  const [running, setRunning] = useState(false)
  const [ready, setReady] = useState(false)
  const [busy, setBusy] = useState(false)
  const [debug, setDebug] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [logs, setLogs] = useState<string[]>([])
  const lifecycle = useRef<AbortController | null>(null)
  const lock = useRef(false)
  useEffect(() => {
    const controller = new AbortController()
    lifecycle.current = controller
    api
      .running(controller.signal)
      .then((value) => {
        if (!controller.signal.aborted) {
          setRunning(value)
          setReady(true)
        }
      })
      .catch((e) => {
        if (!controller.signal.aborted) setError(String(e))
      })
    const unlisten = listen<string>('debug-log', (event) => {
      if (!controller.signal.aborted)
        setLogs((values) =>
          [`[${new Date().toLocaleTimeString()}] ${event.payload}`, ...values].slice(0, 300),
        )
    }).catch(() => () => {})
    return () => {
      controller.abort()
      void unlisten.then((stop) => stop())
      lock.current = false
    }
  }, [])
  const toggle = async (mode: ConnectionMode | null, connected: boolean, blocked: boolean) => {
    const controller = lifecycle.current
    if (!controller || lock.current || !ready || blocked || (!running && (!mode || !connected)))
      return
    lock.current = true
    setBusy(true)
    setError(null)
    try {
      if (running) await api.stop(controller.signal)
      else await api.start(mode!, controller.signal)
      if (!controller.signal.aborted) setRunning(!running)
    } catch (e) {
      if (!controller.signal.aborted) {
        setError(String(e))
        // Reconcile after any ambiguous timeout before enabling another action.
        try {
          const value = await api.running(controller.signal)
          if (!controller.signal.aborted) setRunning(value)
        } catch {
          if (!controller.signal.aborted) setReady(false)
        }
      }
    } finally {
      if (!controller.signal.aborted) {
        lock.current = false
        setBusy(false)
      }
    }
  }
  const toggleDebug = async (enabled: boolean) => {
    const controller = lifecycle.current
    if (!controller) return
    try {
      await api.debug(enabled, controller.signal)
      if (!controller.signal.aborted) setDebug(enabled)
    } catch (e) {
      if (!controller.signal.aborted) setError(String(e))
    }
  }
  return { running, ready, busy, debug, error, logs, toggle, toggleDebug }
}
