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

function save(key: number | undefined): void {
  if (key === undefined || !hasWindow()) return
  positions.set(key, { x: window.scrollX, y: window.scrollY })
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
  window.addEventListener(
    'scroll',
    () => {
      if (ticking) return
      ticking = true
      requestAnimationFrame(() => {
        ticking = false
        save(currentKey())
      })
    },
    { passive: true },
  )
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
 * Applies the pending scroll on the next frame, once the new view's DOM is in
 * place. No pending intent → no-op.
 */
export function applyScroll(): void {
  const intent = pending
  pending = null
  if (!intent || !hasWindow()) return
  const run = (): void => {
    if (intent.kind === 'top') {
      window.scrollTo(0, 0)
      return
    }
    const pos = positions.get(intent.key)
    window.scrollTo(pos?.x ?? 0, pos?.y ?? 0)
  }
  // Restore after paint so the destination view's height exists; fall back to
  // a synchronous apply when rAF is unavailable (jsdom without a shim).
  if (typeof requestAnimationFrame === 'function') requestAnimationFrame(run)
  else run()
}

/** Test seam — reset module state between cases. */
export function __resetScrollRestoration(): void {
  installed = false
  counter = 0
  positions.clear()
  pending = null
}
