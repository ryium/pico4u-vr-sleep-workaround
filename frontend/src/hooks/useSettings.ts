import { useCallback, useEffect, useRef, useState } from 'react'
import { api } from '../lib/api'
import { configWriter } from '../lib/settings'
import type { AppConfig } from '../types'

export function useSettings() {
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [pending, setPending] = useState(0)
  const writer = useRef<ReturnType<typeof configWriter> | null>(null)
  const lifecycle = useRef<AbortController | null>(null)
  useEffect(() => {
    const controller = new AbortController()
    lifecycle.current = controller
    api
      .config(controller.signal)
      .then((value) => {
        if (controller.signal.aborted) return
        writer.current = configWriter(value, (next) => api.save(next, controller.signal))
        setConfig(value)
      })
      .catch((e) => {
        if (!controller.signal.aborted) setError(String(e))
      })
    return () => {
      controller.abort()
      writer.current = null
    }
  }, [])
  const update = useCallback(async (patch: Partial<AppConfig>) => {
    const controller = lifecycle.current
    if (!writer.current || !controller || controller.signal.aborted)
      throw new Error('settings_unavailable')
    setPending((n) => n + 1)
    try {
      const next = await writer.current(patch)
      if (!controller.signal.aborted) {
        setConfig(next)
        setError(null)
      }
      return next
    } catch (e) {
      if (!controller.signal.aborted) setError(String(e))
      throw e
    } finally {
      if (!controller.signal.aborted) setPending((n) => n - 1)
    }
  }, [])
  return { config, update, error, isSaving: pending > 0 }
}
