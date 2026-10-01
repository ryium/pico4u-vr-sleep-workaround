import type { AppConfig } from '../types'
import { serialQueue } from './async'

export function configWriter(initial: AppConfig, save: (config: AppConfig) => Promise<void>) {
  let desired = initial
  let committed = initial
  const enqueue = serialQueue()
  return (patch: Partial<AppConfig>) => {
    // Merge synchronously, including explicit null. Consecutive writes cannot
    // read a stale React render or complete out of order.
    desired = { ...desired, ...patch }
    const snapshot = desired
    return enqueue(async () => {
      try {
        await save(snapshot)
        committed = snapshot
        return snapshot
      } catch (error) {
        // An invalid/failed latest write must not poison every later patch.
        // Already queued snapshots still represent the user's subsequent edits.
        if (desired === snapshot) desired = committed
        throw error
      }
    })
  }
}
