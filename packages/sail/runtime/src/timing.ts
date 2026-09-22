// Lifecycle-bound timing — debounce / interval enrolled on the current
// effect scope. Auto-cleanup on dispose (same mechanism as mount/onCleanup).
// Replaces ambient `ReturnType<typeof setTimeout> | null` ceremony in app code.
//
// Outside a scope, timers still schedule but have no owner (test helpers,
// module-level demos). Callers that need guaranteed teardown must be inside
// setup / an effect / withScope.

import { onCleanup } from './effect-scope'

/**
 * Debounce `fn` by `ms`. Returns a function that delays each call until
 * `ms` of quiet. Clears any pending timer when the current effect scope
 * disposes.
 *
 * @example
 * ```ts
 * const save = debounce((text: string) => api.save(text), 300)
 * onInput((e) => save(e.target.value))
 * ```
 */
export function debounce<Args extends unknown[]>(
  fn: (...args: Args) => void,
  ms: number,
): (...args: Args) => void {
  let timer: ReturnType<typeof setTimeout> | null = null
  const clear = (): void => {
    if (timer !== null) {
      clearTimeout(timer)
      timer = null
    }
  }
  onCleanup(clear)
  return (...args: Args) => {
    clear()
    timer = setTimeout(() => {
      timer = null
      fn(...args)
    }, ms)
  }
}

/**
 * Call `fn` every `ms` milliseconds for the life of the current effect
 * scope. Clears the interval on dispose. No-op when `setInterval` is
 * unavailable (SSR / non-browser hosts).
 *
 * @example
 * ```ts
 * interval(() => ticks.set((n) => n + 1), 1000)
 * ```
 */
export function interval(fn: () => void, ms: number): void {
  if (typeof setInterval === 'undefined') return
  const id = setInterval(fn, ms)
  onCleanup(() => clearInterval(id))
}
