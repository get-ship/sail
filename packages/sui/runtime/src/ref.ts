// Refs — the SUI parallel of Svelte's `bind:this={el}` and React's
// `useRef<T>()`.
//
// A `Ref<T>` is a `Signal<T | null>` slot the runtime writes into when
// the corresponding `bind:this={var}` element mounts, and clears on
// unmount. Authors declare a ref by adding `var: ref(null)` to the
// controller's `scope:` factory; the compiler's `bind_this_target`
// check enforces this shape. The paired template reads it with `var()`
// like any signal.
//
// Refs are write-once-from-runtime, read-many-from-controller —
// authors should never call `.set()` on a ref themselves.

import { signal, isSignal, type Signal } from './signal'

/**
 * Typed handle to a single rendered element. Reads as `null` before
 * the runtime resolves the element and after the boundary unmounts;
 * returns the live element while attached.
 */
export type Ref<T extends Element = HTMLElement> = Signal<T | null>

/**
 * Construct a writable ref slot, seeded with `null`. The runtime
 * fills it from the matching `bind:this={var}` mount; the compiler's
 * `bind_this_target` check requires every `bind:this` target to be
 * declared as `name: ref(null)` in the controller's scope factory.
 */
export function ref<T extends Element = HTMLElement>(_initial: null = null): Ref<T> {
  return signal<T | null>(null)
}

/**
 * `bind:this` setup used by compiler-emitted templates. Reuses an
 * existing writable signal in `scope[varName]` (typically a
 * pre-declared `ref(null)`), otherwise installs a fresh signal seeded
 * with the element.
 */
export function attachRef<T extends Element>(
  scope: Record<string, unknown>,
  varName: string,
  el: T,
): Signal<T | null> {
  const existing = scope[varName]
  if (isSignal(existing)) {
    ;(existing as Signal<T | null>).set(el)
    return existing as Signal<T | null>
  }
  const slot = signal<T | null>(el)
  scope[varName] = slot
  return slot
}
