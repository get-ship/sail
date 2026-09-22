// scroll-restoration — SPA scroll position management for the deck router.
//
// The browser's own `history.scrollRestoration` is unreliable across SPA
// navigations (it restores against DOM that hasn't re-rendered yet). We take
// manual control:
//
//   • Every history entry is stamped with a monotonic key in `history.state`.
//   • The current scroll offset is saved (keyed) continuously (throttled) and
//     again immediately before each navigation.
//   • A fresh push scrolls to the top; back/forward (popstate) restores the
//     target entry's saved offset — applied AFTER the route re-renders.
//   • Nested overflow hosts (docs main, deck route-slot, landing main) are
//     first-class: window.scrollY is often 0 while the real offset lives on
//     an overflow:auto ancestor. Push zeros those hosts; we also save the
//     deepest nested Y for popstate restore when the host still exists.
//
// Wired from `index.ts`: `installScrollRestoration()` once, `onNavigate()` +
// `applyScroll()` around a programmatic `navigate()`, and `onPopState()` +
// `applyScroll()` in the popstate handler. SSR/no-window contexts are no-ops.

const STATE_KEY = '__shipScroll'

interface ScrollState {
  [STATE_KEY]?: number
}
interface Offset {
  x: number
  y: number
  /** Deepest nested overflow host scrollTop (docs/landing scrollports). */
  nestedY?: number
}
type PendingScroll = { kind: 'top' } | { kind: 'restore'; key: number } | null

let installed = false
let counter = 0
const positions = new Map<number, Offset>()
let pending: PendingScroll = null

function hasWindow(): boolean {
  return typeof window !== 'undefined' && !!window.history
}

function keyOf(state: unknown): number | undefined {
  const k = (state as ScrollState | null)?.[STATE_KEY]
  return typeof k === 'number' ? k : undefined
}

function currentKey(): number | undefined {
  return hasWindow() ? keyOf(window.history.state) : undefined
}

/** App root — prefer #root when present (deck host). */
function appRoot(): HTMLElement {
  return (
    (typeof document !== 'undefined' &&
      (document.getElementById('root') as HTMLElement | null)) ||
    document.body
  )
}

/**
 * Overflow hosts under the app that actually scroll (or can). Used to zero
 * on push and to capture nested Y for back/forward.
 */
function nestedScrollHosts(): HTMLElement[] {
  if (typeof document === 'undefined') return []
  const out: HTMLElement[] = []
  const walk = (el: Element): void => {
    if (el instanceof HTMLElement && el !== document.documentElement && el !== document.body) {
      const s = getComputedStyle(el)
      const oy = s.overflowY
      const ox = s.overflowX
      const canY = oy === 'auto' || oy === 'scroll' || oy === 'overlay'
      const canX = ox === 'auto' || ox === 'scroll' || ox === 'overlay'
      if (
        (canY && (el.scrollHeight > el.clientHeight + 1 || el.scrollTop > 0)) ||
        (canX && (el.scrollWidth > el.clientWidth + 1 || el.scrollLeft > 0))
      ) {
        out.push(el)
      }
    }
    for (const c of el.children) walk(c)
  }
  walk(appRoot())
  return out
}

/** Zero every nested overflow host (and window). Call after a new route paints. */
function scrollAllToTop(): void {
  window.scrollTo(0, 0)
  for (const el of nestedScrollHosts()) {
    el.scrollTop = 0
    el.scrollLeft = 0
  }
  // Route slot hosts may not yet report as "scrollable" in the same frame
  // (jsdom / pre-measure). Force-clear any offset under the app root, including
  // the root itself (#root is often the overflow host).
  const root = appRoot()
  const zero = (node: HTMLElement): void => {
    if (node.scrollTop) node.scrollTop = 0
    if (node.scrollLeft) node.scrollLeft = 0
  }
  zero(root)
  root.querySelectorAll('*').forEach((node) => {
    if (node instanceof HTMLElement) zero(node)
  })
}

function deepestNestedY(): number {
  let max = 0
  for (const el of nestedScrollHosts()) {
    if (el.scrollTop > max) max = el.scrollTop
  }
  return max
}

function save(key: number | undefined): void {
  if (key === undefined || !hasWindow()) return
  positions.set(key, {
    x: window.scrollX,
    y: window.scrollY,
    nestedY: deepestNestedY(),
  })
}

/** Ensure the current history entry carries a scroll key; return it. Stamps
 *  the key via `replaceState` (preserving any existing state) when absent. */
function ensureKey(): number {
  const existing = currentKey()
  if (existing !== undefined) return existing
  const key = ++counter
  const state = { ...(window.history.state ?? {}), [STATE_KEY]: key }
  window.history.replaceState(state, '')
  return key
}

/** Build the `history.state` to push/replace for a navigation: a fresh key on
 *  push (so it's a distinct restore target), the existing key on replace. */
function stateForNavigation(replace: boolean): ScrollState {
  const base = (window.history.state as Record<string, unknown> | null) ?? {}
  const key = replace ? ensureKey() : ++counter
  return { ...base, [STATE_KEY]: key }
}

/**
 * Install once. Switches the browser to manual scroll restoration, stamps the
 * initial entry, and starts a throttled scroll saver so every entry's offset
 * is captured even when the user leaves it via the back/forward buttons. Safe
 * to call repeatedly and in non-browser contexts (no-op).
 */
export function installScrollRestoration(): void {
  if (installed || !hasWindow()) return
  installed = true
  try {
    window.history.scrollRestoration = 'manual'
  } catch {
    // Some environments (older Safari, sandboxes) don't allow setting it.
  }
  ensureKey()
  let ticking = false
  const onScroll = (): void => {
    if (ticking) return
    ticking = true
    requestAnimationFrame(() => {
      ticking = false
      save(currentKey())
    })
  }
  // Window scroll (document scrollport).
  window.addEventListener('scroll', onScroll, { passive: true })
  // Nested overflow hosts (docs main, landing main, deck route slot) — capture
  // so back/forward has a nestedY even when window.scrollY stays 0.
  document.addEventListener('scroll', onScroll, { passive: true, capture: true })
}

/**
 * Call from `navigate()` BEFORE the pushState/replaceState. Saves the outgoing
 * offset and records the post-render intent (a push scrolls to top; a replace
 * keeps the current position). Returns the `history.state` to push/replace.
 */
export function onNavigate(replace: boolean): ScrollState {
  if (!hasWindow()) return {}
  save(currentKey())
  const state = stateForNavigation(replace)
  pending = replace ? null : { kind: 'top' }
  return state
}

/**
 * Call from the popstate handler (before applying the route update). Records
 * the intent to restore the destination entry's saved offset.
 */
export function onPopState(): void {
  if (!hasWindow()) return
  const key = currentKey()
  pending = key !== undefined ? { kind: 'restore', key } : { kind: 'top' }
}

/**
 * Call AFTER the route has re-rendered (right after `__shipSetRouteUpdate`).
 * Applies the pending scroll on the next frame, once the new route's DOM is in
 * place. No pending intent → no-op.
 *
 * Push always zeros **nested** overflow hosts first — those are the real
 * scrollports after the docs/landing height-chain (window.scrollY is often 0).
 */
export function applyScroll(): void {
  const intent = pending
  pending = null
  if (!intent || !hasWindow()) return
  const run = (): void => {
    if (intent.kind === 'top') {
      scrollAllToTop()
      return
    }
    // Restore: clear leftover nested offset from the page we left, then apply
    // saved window + deepest nested Y onto current scroll hosts.
    const pos = positions.get(intent.key)
    scrollAllToTop()
    window.scrollTo(pos?.x ?? 0, pos?.y ?? 0)
    const nestedY = pos?.nestedY ?? 0
    if (nestedY > 0) {
      // Prefer the tallest scrollable content host (usually docs/landing main).
      const hosts = nestedScrollHosts().sort(
        (a, b) => b.scrollHeight - b.clientHeight - (a.scrollHeight - a.clientHeight),
      )
      const target = hosts[0]
      if (target) target.scrollTop = nestedY
    }
  }
  // Restore after paint so the destination route's height exists; fall back to
  // a synchronous apply when rAF is unavailable (jsdom without a shim).
  // Double-rAF: first frame mounts the leaf; second has measured scrollHeight
  // on overflow hosts so nestedY restore and top-zero stick.
  if (typeof requestAnimationFrame === 'function') {
    requestAnimationFrame(() => requestAnimationFrame(run))
  } else run()
}

/** Test seam — reset module state between cases. */
export function __resetScrollRestoration(): void {
  installed = false
  counter = 0
  positions.clear()
  pending = null
}
