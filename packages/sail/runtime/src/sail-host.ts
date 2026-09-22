/**
 * SailHost — opaque mount-host handle for Pure product code.
 *
 * Design (JOB-E2 / SPEC-TYPE-GATE-1): capability firewall, not a platform
 * abstraction. On the web runtime the value **is** an HTMLElement; the safety
 * property is the type surface authors see in Pure — zero methods, zero
 * properties, identity only. Pass to actions and runtime APIs (focus, portal
 * anchor, media mount, theme.applyScope). Do not cast to DOM types in Pure.
 *
 * Framework modules unwrap via `unwrapSailHost` in `./sail-host-dom`
 * (`@ship/sail-runtime/internal`). This module stays Pure-portable
 * (`lib: ESNext` only — no DOM type names).
 */
declare const __sailHostBrand: unique symbol

/** Opaque branded host identity — no callables, no readable props for authors. */
export type SailHost = { readonly [__sailHostBrand]: typeof __sailHostBrand }

/**
 * Schedule work on the next animation frame (microtask fallback).
 * Pure product code must not name `requestAnimationFrame` — use this.
 * Implemented via structural `globalThis` check (no DOM lib name required).
 */
export function scheduleFrame(fn: () => void): void {
  const g = globalThis as { requestAnimationFrame?: (cb: () => void) => unknown }
  if (typeof g.requestAnimationFrame === 'function') {
    g.requestAnimationFrame(() => fn())
  } else {
    queueMicrotask(fn)
  }
}

/** Coerce a handed action `node` (or other host value) to SailHost (identity on web). */
export function asSailHost(node: unknown): SailHost {
  return node as SailHost
}
