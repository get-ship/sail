// persisted — a Signal backed by web storage.
//
// SSR-safe: server render uses `initial`; storage hydrates on mount.
// Writes are debounced via lifecycle-bound timing so rapid edits do not
// thrash storage. The ambient door is localStorage/sessionStorage —
// authors use this instead.

import { debounce } from './timing'
import { effect, signal, type Signal } from './signal'
import { mount as mountAmbient, onCleanup, currentScope } from './effect-scope'

export type PersistedOpts = {
  /** Which web storage. Default `'local'`. */
  storage?: 'local' | 'session'
  /** Debounce window for writes (ms). Default `100`. */
  debounceMs?: number
}

function storageOf(kind: 'local' | 'session'): Storage | null {
  if (typeof window === 'undefined') return null
  try {
    return kind === 'session' ? window.sessionStorage : window.localStorage
  } catch {
    // Opaque origins / disabled storage throw on the getter.
    return null
  }
}

/**
 * A writable signal that rehydrates from and writes through to web storage
 * under `name` (JSON-serialized).
 *
 * - **SSR / no storage:** behaves as `signal(initial)`.
 * - **Client:** reads storage on mount, then debounces writes on every
 *   subsequent `.set`.
 *
 * @example
 * ```ts
 * const theme = persisted('ship-site-theme', 'ship')
 * theme.set('console') // signal updates; storage write is debounced
 * ```
 */
export function persisted<T>(
  name: string,
  initial: T,
  opts?: PersistedOpts,
): Signal<T> {
  const kind = opts?.storage ?? 'local'
  const debounceMs = opts?.debounceMs ?? 100
  const sig = signal(initial)

  const write = debounce((value: T) => {
    const store = storageOf(kind)
    if (!store) return
    try {
      store.setItem(name, JSON.stringify(value))
    } catch {
      /* quota / private mode */
    }
  }, debounceMs)

  const hydrateAndTrack = (): void => {
    const store = storageOf(kind)
    if (store) {
      try {
        const raw = store.getItem(name)
        if (raw != null && raw !== '') {
          sig.set(JSON.parse(raw) as T)
        }
      } catch {
        /* corrupt JSON — keep initial */
      }
    }
    // Track after hydrate so the hydrate write does not immediately re-serialize.
    let first = true
    effect(() => {
      const v = sig()
      if (first) {
        first = false
        return
      }
      write(v)
    })
  }

  // Prefer mount (DOM-connected) when inside a controller/component setup so
  // SSR paint keeps `initial` and the client hydrates once. Outside a scope
  // (unit tests), hydrate immediately when storage is available.
  if (currentScope()) {
    try {
      mountAmbient(() => {
        hydrateAndTrack()
      })
    } catch {
      hydrateAndTrack()
    }
  } else if (typeof window !== 'undefined') {
    hydrateAndTrack()
  }

  // If the scope dies before mount runs, drop any pending write.
  onCleanup(() => {
    /* debounce already enrolled its own cleanup */
  })

  return sig
}
