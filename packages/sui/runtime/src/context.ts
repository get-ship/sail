// Cross-component reactive context.
//
// `shared(signal)` marks a signal as published to descendants. The
// host component calls `provide(host, key, scope)` once at mount with
// its scope; the runtime extracts marked signals for publication.
// Descendants call `useShared(host, key)` to look up the nearest
// ancestor providing under that key, walking DOM parents and shadow
// boundaries.

import type { Signal } from './signal'

const SHARED = Symbol('sui.shared')

/**
 * Mark a signal as shared with descendant components. Wraps the
 * signal — same callable signal, plus a flag the runtime reads to
 * decide what to publish via `provide`.
 *
 * @example
 * ```ts
 * scope: () => ({
 *   mode: shared(signal<'find' | 'command'>('find')),
 *   selectedIdx: signal(0),  // local — not published
 * })
 * ```
 */
export function shared<T>(sig: Signal<T>): Signal<T> {
  ;(sig as Signal<T> & { [SHARED]?: true })[SHARED] = true
  return sig
}

/**
 * Test whether a signal was marked with `shared(...)`.
 */
export function isShared(sig: unknown): boolean {
  return !!(sig as { [SHARED]?: true } | null)?.[SHARED]
}

/**
 * Publish a context value on `host` under `key`. Descendants reach it
 * via `useShared(host, key)`. Implementation: a single property on the
 * host element. The host's lifecycle owns it — when the element is
 * removed the property goes with it.
 */
export function provide<T>(host: Element, key: string, value: T): void {
  ;(host as unknown as Record<string, unknown>)[`__sui_ctx_${key}`] = value
}

/**
 * Look up a context published by an ancestor. Walks `host.parentElement`
 * and crosses shadow-root boundaries via `getRootNode().host`.
 *
 * Returns `null` when no harness is found — consumer guards with
 * `{#if bar}` or controller-side null checks.
 */
export function useShared<T>(host: Element, key: string): T | null {
  let cur: Element | null = host
  while (cur) {
    const v = (cur as unknown as Record<string, unknown>)[`__sui_ctx_${key}`]
    if (v !== undefined) return v as T
    const parent: Element | null = cur.parentElement
    if (parent) {
      cur = parent
      continue
    }
    const root = cur.getRootNode()
    cur = root instanceof ShadowRoot ? root.host : null
  }
  return null
}

/**
 * Filter a scope object to entries whose values were marked
 * `shared(...)`. The compiler-emitted setup calls this with the full
 * scope at mount, then `provide`s the result.
 */
export function pickShared<T extends Record<string, unknown>>(scope: T): Partial<T> {
  const out: Partial<T> = {}
  for (const k in scope) {
    if (Object.prototype.hasOwnProperty.call(scope, k) && isShared(scope[k])) {
      out[k] = scope[k]
    }
  }
  return out
}
