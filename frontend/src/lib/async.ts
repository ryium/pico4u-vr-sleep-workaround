export function abortError() {
  return new DOMException('Aborted', 'AbortError')
}
export function delay(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal.aborted) {
      reject(abortError())
      return
    }
    const abort = () => {
      clearTimeout(timer)
      reject(abortError())
    }
    const timer = setTimeout(() => {
      signal.removeEventListener('abort', abort)
      resolve()
    }, ms)
    signal.addEventListener('abort', abort, { once: true })
  })
}

// Timing out a Tauri invoke does not cancel Rust. Keep the underlying operation
// in the serial queue until Rust's own deadline has ended it.
export function serialQueue() {
  let tail: Promise<unknown> = Promise.resolve()
  return <T>(task: () => Promise<T>, signal?: AbortSignal): Promise<T> => {
    const next = tail.then(() => {
      if (signal?.aborted) throw abortError()
      return task()
    })
    tail = next.catch(() => {})
    return next
  }
}

export async function withTimeout<T>(
  promise: Promise<T>,
  ms: number,
  signal: AbortSignal,
): Promise<T> {
  if (signal.aborted) throw abortError()
  let timer: ReturnType<typeof setTimeout> | undefined
  let abort: () => void = () => {}
  try {
    return await Promise.race([
      promise,
      new Promise<T>((_, reject) => {
        timer = setTimeout(() => reject(new Error('connection_timeout')), ms)
        abort = () => reject(abortError())
        signal.addEventListener('abort', abort, { once: true })
      }),
    ])
  } finally {
    clearTimeout(timer)
    signal.removeEventListener('abort', abort)
  }
}

export async function retry<T>(
  task: () => Promise<T>,
  signal: AbortSignal,
  onAttempt: (n: number) => void,
  attempts = 3,
): Promise<T> {
  for (let attempt = 1; ; attempt++) {
    if (signal.aborted) throw abortError()
    onAttempt(attempt)
    try {
      return await task()
    } catch (error) {
      if (signal.aborted || attempt >= attempts) throw error
      await delay(2000, signal)
    }
  }
}

export function poll<T>(
  task: () => Promise<T>,
  receive: (value: T) => void,
  fail: (error: unknown) => void,
  signal: AbortSignal,
  ms = 2000,
) {
  void (async () => {
    while (!signal.aborted) {
      try {
        const value = await task()
        if (!signal.aborted) receive(value)
      } catch (error) {
        if (!signal.aborted) fail(error)
      }
      if (signal.aborted) break
      try {
        await delay(ms, signal)
      } catch {
        break
      }
    }
  })()
}
