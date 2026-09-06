import { afterEach, describe, expect, it, vi } from 'vitest'
import { delay, poll, retry, serialQueue, withTimeout } from './async'

afterEach(() => vi.useRealTimers())
describe('async lifecycle', () => {
  it('clears timers on resolution, rejection, timeout and abort', async () => {
    vi.useFakeTimers()
    const c = new AbortController()
    expect(await withTimeout(Promise.resolve(4), 5000, c.signal)).toBe(4)
    await expect(withTimeout(Promise.reject(new Error('failed')), 5000, c.signal)).rejects.toThrow(
      'failed',
    )
    expect(vi.getTimerCount()).toBe(0)
    const timed = expect(withTimeout(new Promise(() => {}), 100, c.signal)).rejects.toThrow(
      'connection_timeout',
    )
    await vi.advanceTimersByTimeAsync(100)
    await timed
    const waiting = expect(delay(2000, c.signal)).rejects.toMatchObject({ name: 'AbortError' })
    c.abort()
    await waiting
    expect(vi.getTimerCount()).toBe(0)
  })
  it('retries at most three times and stops during retry delay', async () => {
    vi.useFakeTimers()
    const c = new AbortController()
    const task = vi.fn().mockRejectedValue(new Error('offline'))
    const result = expect(retry(task, c.signal, () => {})).rejects.toThrow('offline')
    await vi.advanceTimersByTimeAsync(4000)
    await result
    expect(task).toHaveBeenCalledTimes(3)
    task.mockClear()
    const stopped = expect(retry(task, c.signal, () => {})).rejects.toMatchObject({
      name: 'AbortError',
    })
    await vi.advanceTimersByTimeAsync(1)
    c.abort()
    await stopped
    await vi.advanceTimersByTimeAsync(10000)
    expect(task).toHaveBeenCalledTimes(1)
  })
  it('does not overlap polls or publish results after disposal', async () => {
    vi.useFakeTimers()
    const c = new AbortController()
    let resolve!: (value: string) => void
    const task = vi.fn(
      () =>
        new Promise<string>((r) => {
          resolve = r
        }),
    )
    const receive = vi.fn()
    poll(task, receive, vi.fn(), c.signal, 10)
    await vi.advanceTimersByTimeAsync(100)
    expect(task).toHaveBeenCalledTimes(1)
    c.abort()
    resolve('stale')
    await vi.advanceTimersByTimeAsync(100)
    expect(receive).not.toHaveBeenCalled()
    expect(task).toHaveBeenCalledTimes(1)
    expect(vi.getTimerCount()).toBe(0)
  })
  it('reports polling errors and recovers on the next check', async () => {
    vi.useFakeTimers()
    const c = new AbortController()
    const task = vi
      .fn()
      .mockRejectedValueOnce(new Error('unavailable'))
      .mockResolvedValue('connected')
    const receive = vi.fn(),
      fail = vi.fn()
    poll(task, receive, fail, c.signal, 10)
    await vi.advanceTimersByTimeAsync(0)
    expect(fail).toHaveBeenCalledOnce()
    await vi.advanceTimersByTimeAsync(10)
    expect(receive).toHaveBeenCalledWith('connected')
    c.abort()
  })
  it('keeps a timed-out underlying command serialized and skips cancelled work', async () => {
    vi.useFakeTimers()
    const queue = serialQueue(),
      c = new AbortController()
    let finish!: () => void
    const first = queue(
      () =>
        new Promise<void>((r) => {
          finish = r
        }),
    )
    const timed = expect(withTimeout(first, 10, c.signal)).rejects.toThrow('connection_timeout')
    const second = vi.fn().mockResolvedValue(2)
    const queued = expect(queue(second, c.signal)).rejects.toMatchObject({ name: 'AbortError' })
    await vi.advanceTimersByTimeAsync(10)
    await timed
    expect(second).not.toHaveBeenCalled()
    c.abort()
    finish()
    await queued
    expect(second).not.toHaveBeenCalled()
  })
})
