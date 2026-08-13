/**
 * Pure-safe host identity-property install (SPEC-L2-PACKAGE-1 P1).
 *
 * Controllers must not name `Object.defineProperty` on the mount host for
 * identity props (`src`, `value`, `canvas`, …). This helper installs a
 * configurable getter/setter on the underlying element and returns a
 * disposer that restores the previous descriptor.
 */

import type { SailHost } from './sail-host'
import { unwrapSailHost } from './sail-host-dom'

export type HostPropertyDesc = {
  get: () => unknown
  set?: (value: unknown) => void
}

/**
 * Install an identity property on `host`. Returns a disposer that removes
 * the property (or restores the prior descriptor).
 */
export function defineHostProperty(
  host: SailHost,
  name: string,
  desc: HostPropertyDesc,
): () => void {
  const el = unwrapSailHost(host) as unknown as Record<string, unknown>
  const existing = Object.getOwnPropertyDescriptor(el, name)
  Object.defineProperty(el, name, {
    configurable: true,
    enumerable: true,
    get: desc.get,
    set: desc.set
      ? (v: unknown) => {
          desc.set!(v)
        }
      : undefined,
  })
  return () => {
    if (existing) {
      Object.defineProperty(el, name, existing)
    } else {
      try {
        delete el[name]
      } catch {
        el[name] = undefined
      }
    }
  }
}

/** Read a host attribute (Pure-safe — no HTMLElement methods on authors). */
export function hostGetAttr(host: SailHost | null | undefined, name: string): string | null {
  if (host == null) return null
  const el = unwrapSailHost(host) as { getAttribute?: (n: string) => string | null }
  return el.getAttribute?.(name) ?? null
}

/** Presence check for a host attribute. */
export function hostHasAttr(host: SailHost | null | undefined, name: string): boolean {
  if (host == null) return false
  const el = unwrapSailHost(host) as { hasAttribute?: (n: string) => boolean }
  return el.hasAttribute?.(name) === true
}

/** Write/remove a host attribute (`null`/`false` removes; `true` = presence). */
export function hostSetAttr(
  host: SailHost | null | undefined,
  name: string,
  value: string | boolean | null | undefined,
): void {
  if (host == null) return
  const el = unwrapSailHost(host) as {
    setAttribute?: (n: string, v: string) => void
    removeAttribute?: (n: string) => void
  }
  if (value === null || value === undefined || value === false) {
    el.removeAttribute?.(name)
    return
  }
  if (value === true) {
    el.setAttribute?.(name, '')
    return
  }
  el.setAttribute?.(name, value)
}

/** Dispatch a named event on a handed node (Pure-safe — no Event constructor in authors). */
export function hostDispatch(
  node: unknown,
  type: string,
  opts?: { bubbles?: boolean; composed?: boolean; cancelable?: boolean; detail?: unknown },
): boolean {
  if (node == null) return false
  const el = unwrapSailHost(node as SailHost) as { dispatchEvent?: (e: unknown) => boolean }
  const g = globalThis as {
    CustomEvent?: new (type: string, init?: object) => unknown
    Event?: new (type: string, init?: object) => unknown
  }
  const ev =
    opts && 'detail' in opts && typeof g.CustomEvent === 'function'
      ? new g.CustomEvent(type, opts)
      : typeof g.Event === 'function'
        ? new g.Event(type, opts)
        : null
  if (ev == null) return false
  return el.dispatchEvent?.(ev) ?? false
}

/**
 * Observe content-box size of a handed action `node` (or SailHost).
 * Pure-safe stand-in for naming `ResizeObserver` in product modules.
 */
export function observeContentBox(
  node: unknown,
  onBox: (width: number, height: number) => void,
): () => void {
  const el = node as { getBoundingClientRect?: () => { width: number; height: number } } | null
  if (el == null) return () => {}

  const g = globalThis as {
    ResizeObserver?: new (cb: (entries: ReadonlyArray<{ contentRect: { width: number; height: number } }>) => void) => {
      observe: (t: unknown) => void
      disconnect: () => void
    }
  }

  if (typeof g.ResizeObserver === 'function') {
    const ro = new g.ResizeObserver((entries) => {
      const box = entries[entries.length - 1]?.contentRect
      if (!box) return
      onBox(box.width, box.height)
    })
    ro.observe(el)
    // Seed once from current box when available.
    const seed = el.getBoundingClientRect?.()
    if (seed && seed.width > 0) onBox(seed.width, seed.height)
    return () => ro.disconnect()
  }

  // Fallback: single seed measure (no live observe).
  const seed = el.getBoundingClientRect?.()
  if (seed) onBox(seed.width, seed.height)
  return () => {}
}

// ── Document-level Pure-safe helpers (mermaid theme, boundary templates, …) ─

type DocLike = {
  documentElement?: HTMLElement & { dataset?: DOMStringMap }
  body?: HTMLElement
  getElementById?: (id: string) => HTMLElement | null
  createElement?: (tag: string) => HTMLElement
  addEventListener?: (t: string, h: EventListener) => void
  removeEventListener?: (t: string, h: EventListener) => void
}

function doc(): DocLike | null {
  const g = globalThis as { document?: DocLike }
  return g.document ?? null
}

/** True when `<html mode="dark">` (or equivalent theme dark). */
export function hostIsDarkMode(): boolean {
  const root = doc()?.documentElement
  return root?.getAttribute?.('mode') === 'dark'
}

/** Read a CSS custom property from the document element. */
export function hostCssVar(name: string): string {
  const root = doc()?.documentElement
  if (!root) return ''
  const g = globalThis as { getComputedStyle?: (el: Element) => { getPropertyValue(n: string): string } }
  return g.getComputedStyle?.(root)?.getPropertyValue(name).trim() ?? ''
}

/** `document.getElementById` without naming document. */
export function hostFindById(id: string): HTMLElement | null {
  return doc()?.getElementById?.(id) ?? null
}

/** Create an element (boundary template promote, etc.). */
export function hostCreateElement(tag: string): HTMLElement {
  const el = doc()?.createElement?.(tag)
  if (!el) throw new Error(`hostCreateElement(${tag}): document unavailable`)
  return el
}

/** Observe attribute changes on the document element; returns disposer. */
export function hostObserveRootAttrs(attrs: string[], cb: () => void): () => void {
  const root = doc()?.documentElement
  if (!root) return () => {}
  const g = globalThis as {
    MutationObserver?: new (cb: () => void) => {
      observe: (t: Node, o: MutationObserverInit) => void
      disconnect: () => void
    }
  }
  if (typeof g.MutationObserver !== 'function') return () => {}
  const mo = new g.MutationObserver(cb)
  mo.observe(root, { attributes: true, attributeFilter: attrs })
  return () => mo.disconnect()
}

/** Listen on document for a named event; returns disposer. */
export function hostListenDocument(event: string, handler: () => void): () => void {
  const d = doc()
  if (!d?.addEventListener) return () => {}
  d.addEventListener(event, handler as EventListener)
  return () => d.removeEventListener?.(event, handler as EventListener)
}

/** Observe attribute changes on a handed host/node; returns disposer. */
export function observeHostAttrs(
  node: unknown,
  attrs: string[],
  cb: () => void,
): () => void {
  if (node == null) return () => {}
  const g = globalThis as {
    MutationObserver?: new (cb: () => void) => {
      observe: (t: unknown, o: { attributes: boolean; attributeFilter: string[] }) => void
      disconnect: () => void
    }
  }
  if (typeof g.MutationObserver !== 'function') return () => {}
  const mo = new g.MutationObserver(cb)
  mo.observe(node, { attributes: true, attributeFilter: attrs })
  return () => mo.disconnect()
}
