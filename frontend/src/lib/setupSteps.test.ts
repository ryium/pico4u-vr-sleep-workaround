import { describe, expect, it } from 'vitest'
import { INITIAL_SETUP_STEPS, updateSetupSteps, type SetupProgressPayload } from './setupSteps'

describe('setupSteps reducer', () => {
  it('starts with initial steps in idle state', () => {
    expect(INITIAL_SETUP_STEPS).toHaveLength(5)
    expect(INITIAL_SETUP_STEPS.every((s) => s.state === 'idle')).toBe(true)
  })

  it('updates target step correctly', () => {
    const payload: SetupProgressPayload = {
      step: 'usb',
      state: 'ok',
      detail: null,
    }
    const updated = updateSetupSteps(INITIAL_SETUP_STEPS, payload)
    const usbStep = updated.find((s) => s.id === 'usb')
    expect(usbStep?.state).toBe('ok')

    const wifiStep = updated.find((s) => s.id === 'wifi')
    expect(wifiStep?.state).toBe('idle')
  })

  it('records failed state and detail', () => {
    const payload: SetupProgressPayload = {
      step: 'wifi',
      state: 'failed',
      detail: 'wifi_disabled',
    }
    const updated = updateSetupSteps(INITIAL_SETUP_STEPS, payload)
    const wifiStep = updated.find((s) => s.id === 'wifi')
    expect(wifiStep?.state).toBe('failed')
    expect(wifiStep?.detail).toBe('wifi_disabled')
  })
})
