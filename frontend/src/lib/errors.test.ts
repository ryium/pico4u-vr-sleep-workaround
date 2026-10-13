import { describe, expect, it } from 'vitest'
import { toAppError, formatErrorLog, type ErrorCode } from './errors'
import i18n from '../i18n'

describe('errors classification and formatting', () => {
  it('converts AppError objects correctly', () => {
    const err = toAppError({ code: 'usb_unauthorized', detail: 'device unauthorized' })
    expect(err).toEqual({ code: 'usb_unauthorized', detail: 'device unauthorized' })
  })

  it('converts raw string errors to structured AppError', () => {
    expect(toAppError('connection_timeout')).toEqual({
      code: 'timeout',
      detail: 'connection_timeout',
    })
    expect(toAppError('pico_not_connected')).toEqual({
      code: 'usb_not_found',
      detail: 'pico_not_connected',
    })
    expect(toAppError('device unauthorized.\nPlease check...')).toEqual({
      code: 'usb_unauthorized',
      detail: 'device unauthorized.\nPlease check...',
    })
    expect(toAppError('device offline')).toEqual({
      code: 'usb_offline',
      detail: 'device offline',
    })
    expect(toAppError('Failed to start ADB server')).toEqual({
      code: 'adb_unavailable',
      detail: 'Failed to start ADB server',
    })
    expect(toAppError('something unexpected')).toEqual({
      code: 'unknown',
      detail: 'something unexpected',
    })
  })

  it('formats error logs with or without detail', () => {
    const err = { code: 'usb_not_found' as ErrorCode, detail: 'check cable' }
    const titleOnly = formatErrorLog(
      err,
      (k) => (k === 'errors.usb_not_found.title' ? '見つかりません' : k),
      false,
    )
    expect(titleOnly).toBe('見つかりません')

    const withDetail = formatErrorLog(
      err,
      (k) => (k === 'errors.usb_not_found.title' ? '見つかりません' : k),
      true,
    )
    expect(withDetail).toBe('見つかりません (check cable)')
  })

  it('covers all ErrorCode values in both Japanese and English i18n resources', () => {
    const codes: ErrorCode[] = [
      'adb_unavailable',
      'timeout',
      'already_running',
      'invalid_ip',
      'multiple_pico',
      'usb_not_found',
      'usb_unauthorized',
      'usb_offline',
      'not_pico',
      'wifi_disabled',
      'wifi_not_connected',
      'tcpip_failed',
      'wireless_refused',
      'wireless_unreachable',
      'wireless_unauthorized',
      'wireless_not_found',
      'device_lost',
      'unknown',
    ]

    for (const lang of ['ja', 'en']) {
      for (const code of codes) {
        const title = i18n.t(`errors.${code}.title`, { lng: lang })
        expect(title).not.toBe(`errors.${code}.title`)
        expect(title.length).toBeGreaterThan(0)

        const hints = i18n.t(`errors.${code}.hints`, {
          lng: lang,
          returnObjects: true,
        }) as string[]
        expect(Array.isArray(hints)).toBe(true)
        expect(hints.length).toBeGreaterThan(0)
      }
    }
  })
})
