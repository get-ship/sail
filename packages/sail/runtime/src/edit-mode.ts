// edit-mode.ts — per-route edit-mode UI state.
//
// Route authoring (the per-route "edit mode") needs a single, shared bit
// per (project, route) pair so the shell can decide whether to render the
// dev panel + tabs and any route can read the same state without going
// through chrome events.
//
// State model
//   - Each (project, route) has an independent boolean signal.
//   - Signals are in-memory only — they survive SPA navigation via the
//     memoized SIGNALS map but reset on hard reload. Persistence was
//     previously held in localStorage (`ship.edit-mode.<project>.<view>`)
//     but a single accidental Alt+E hit on a built-in route (e.g.
//     /artifacts) would stick forever and re-open the dev panel on every
//     visit. Per-tab UI state doesn't survive reload; the user re-enters
//     edit mode explicitly when they want it.
//   - Entering/exiting calls the host's `editModeTransport` (HostContext)
//     so the runtime names no daemon URL — the host wires it to its
//     route-service enter/exit RPC, or omits it (the toggle still works
//     locally). No `fetch`, no session-key, no /api/v1 from the runtime.
//
// The signals are memoized: calling `editModeForRoute('foo')` twice
// returns the same Signal instance, so multiple consumers in the shell
// + the active route auto-update together.

import { signal, type Signal } from './signal'
import { hostContext } from './host-context'

const SIGNALS = new Map<string, Signal<boolean>>()

function spaceId(): string {
  return hostContext().spaceId ?? 'default'
}

function signalKey(name: string): string {
  return `${spaceId()}.${name}`
}

// One-shot migration: purge legacy `ship.edit-mode.*` keys from
// localStorage. Pre-fix, edit-mode persisted across browser sessions —
// users who accidentally hit Alt+E on a built-in route (e.g. /artifacts)
// would re-enter edit mode on every visit. Clearing here is idempotent
// and cheap.
let migrated = false
function purgeLegacyPersistence(): void {
  if (migrated) return
  migrated = true
  if (typeof localStorage === 'undefined') return
  try {
    const stale: string[] = []
    for (let i = 0; i < localStorage.length; i++) {
      const k = localStorage.key(i)
      if (k && k.startsWith('ship.edit-mode.')) stale.push(k)
    }
    for (const k of stale) localStorage.removeItem(k)
  } catch { /* best-effort */ }
}

function postLifecycle(name: string, action: 'enter' | 'exit'): void {
  // The host owns the edit-mode lifecycle transport (its route-service
  // enter/exit RPC). The runtime names no daemon URL — it calls the
  // injected callback, or no-ops when no host registered one.
  void hostContext().editModeTransport?.(name, action)
}

/** Return the (memoized) Signal<boolean> for whether edit-mode is on
 *  for `name`. Subsequent calls with the same name return the same
 *  signal instance. Always starts OFF on first access; a hard reload
 *  resets every route to off. */
export function editModeForRoute(name: string): Signal<boolean> {
  purgeLegacyPersistence()
  const k = signalKey(name)
  const existing = SIGNALS.get(k)
  if (existing) return existing
  const sig = signal<boolean>(false)
  SIGNALS.set(k, sig)
  return sig
}

/** Flip edit-mode on/off for `name`. Persists locally, fires the
 *  lifecycle POST, and dispatches `sail:edit-mode:change` on document so
 *  shell chrome (which doesn't run inside a tracked effect) can react. */
export function toggleEditMode(name: string): boolean {
  const sig = editModeForRoute(name)
  const next = !sig()
  setEditMode(name, next)
  return next
}

/** Set edit-mode explicitly. No-op when already at the target value. */
export function setEditMode(name: string, on: boolean): void {
  const sig = editModeForRoute(name)
  if (sig() === on) return
  sig.set(on)
  void postLifecycle(name, on ? 'enter' : 'exit')
  if (typeof document !== 'undefined') {
    document.dispatchEvent(new CustomEvent('sail:edit-mode:change', {
      detail: { view: name, on },
      bubbles: true,
    }))
  }
}

// ── Annotation mode (sub-toggle inside edit mode) ────────────────────
//
// Annotation mode is a SECOND boolean per (project, route), only
// meaningful when edit-mode is also on. ON ⇒ the overlay binds
// hover/click handlers so the user picks elements to annotate; OFF ⇒
// the route is interactive normally even though the dev panel + tabs
// are still in edit-mode shape.
//
// Session-only: not persisted to localStorage so a reload always
// starts annotation mode off (per the design call D2 — user wants
// each edit session to start clean).

const ANNOTATION_SIGNALS = new Map<string, Signal<boolean>>()

export function annotationModeForRoute(name: string): Signal<boolean> {
  const k = `ship.annotation-mode.${spaceId()}.${name}`
  const existing = ANNOTATION_SIGNALS.get(k)
  if (existing) return existing
  const sig = signal<boolean>(false)
  ANNOTATION_SIGNALS.set(k, sig)
  return sig
}

export function toggleAnnotationMode(name: string): boolean {
  const sig = annotationModeForRoute(name)
  const next = !sig()
  setAnnotationMode(name, next)
  return next
}

export function setAnnotationMode(name: string, on: boolean): void {
  const sig = annotationModeForRoute(name)
  if (sig() === on) return
  // Annotation mode requires edit mode on; refuse to flip on otherwise.
  if (on && !editModeForRoute(name)()) return
  sig.set(on)
  if (typeof document !== 'undefined') {
    document.dispatchEvent(new CustomEvent('sail:annotation-mode:change', {
      detail: { view: name, on },
      bubbles: true,
    }))
  }
}
