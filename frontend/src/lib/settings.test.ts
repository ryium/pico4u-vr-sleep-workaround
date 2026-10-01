import { expect, it, vi } from 'vitest'
import { configWriter } from './settings'
import { isConnected } from '../types'
import type { AppConfig } from '../types'

const initial: AppConfig = {
  dim_delay_hours: 1,
  keep_awake_interval_secs: 3,
  ip_address: '',
  last_connection_mode: 'wired',
}
it('merges consecutive saves and preserves explicit null in order', async () => {
  let release!: () => void
  const save = vi
    .fn()
    .mockImplementationOnce(
      () =>
        new Promise<void>((r) => {
          release = r
        }),
    )
    .mockResolvedValue(undefined)
  const update = configWriter(initial, save)
  const first = update({ dim_delay_hours: 2 })
  const second = update({ keep_awake_interval_secs: 5, last_connection_mode: null })
  await Promise.resolve()
  expect(save).toHaveBeenCalledTimes(1)
  release()
  await first
  await second
  expect(save.mock.calls[1][0]).toEqual({
    ...initial,
    dim_delay_hours: 2,
    keep_awake_interval_secs: 5,
    last_connection_mode: null,
  })
})
it('surfaces save failure without poisoning subsequent valid edits', async () => {
  const save = vi.fn().mockRejectedValueOnce(new Error('disk')).mockResolvedValue(undefined)
  const update = configWriter(initial, save)
  await expect(update({ dim_delay_hours: 2 })).rejects.toThrow('disk')
  await expect(update({ keep_awake_interval_secs: 4 })).resolves.toMatchObject({
    dim_delay_hours: 1,
    keep_awake_interval_secs: 4,
  })
})
it('never treats loading, failed queries or the other transport as connected', () => {
  expect(isConnected({ kind: 'loading' }, 'wired')).toBe(false)
  expect(isConnected({ kind: 'error', error: 'timeout' }, 'wireless')).toBe(false)
  const result = {
    kind: 'success' as const,
    status: {
      is_usb_connected: true,
      is_adb_tcp_connected: false,
      is_wifi_ip_available: true,
      wifi_ip: '192.168.1.4',
    },
  }
  expect(isConnected(result, 'wireless')).toBe(false)
  expect(isConnected(result, 'wired')).toBe(true)
  expect(isConnected(result, null)).toBe(false)
})
