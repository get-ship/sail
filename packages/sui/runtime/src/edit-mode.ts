// edit-mode.ts — per-view edit-mode UI state.
//
// View authoring (the per-view "edit mode") needs a single, shared bit
// per (project, view) pair so the shell can decide whether to render the
// dev panel + tabs and any view can read the same state without going
// through chrome events.
//
// State model
//   - Each (project, view) has an independent boolean signal.
//   - Signals are in-memory only — they survive SPA navigation via the
//     memoized SIGNALS map but reset on hard reload. Persistence was
//     previously held in localStorage (`ship.edit-mode.<project>.<view>`)
//     but a single accidental Alt+E hit on a built-in view (e.g.
//     /artifacts) would stick forever and re-open the dev panel on every
//     visit. Per-tab UI state doesn't survive reload; the user re-enters
//     edit mode explicitly when they want it.
//   - Entering/exiting calls the host's `editModeTransport` (HostContext)
//     so the runtime names no daemon URL — the host wires it to its
//     view-service enter/exit RPC, or omits it (the toggle still works
//     locally). No `fetch`, no session-key, no /api/v1 from the runtime.
//
// The signals are memoized: calling `editModeForView('foo')` twice
// returns the same Signal instance, so multiple consumers in the shell
// + the active view auto-update together.

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
// users who accidentally hit Alt+E on a built-in view (e.g. /artifacts)
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
  // The host owns the edit-mode lifecycle transport (its view-service
  // enter/exit RPC). The runtime names no daemon URL — it calls the
  // injected callback, or no-ops when no host registered one.
  void hostContext().editModeTransport?.(name, action)
}

/** Return the (memoized) Signal<boolean> for whether edit-mode is on
 *  for `name`. Subsequent calls with the same name return the same
 *  signal instance. Always starts OFF on first access; a hard reload
 *  resets every view to off. */
export function editModeForView(name: string): Signal<boolean> {
  purgeLegacyPersistence()
  const k = signalKey(name)
  const existing = SIGNALS.get(k)
  if (existing) return existing
  const sig = signal<boolean>(false)
  SIGNALS.set(k, sig)
  return sig
}

/** Flip edit-mode on/off for `name`. Persists locally, fires the
 *  lifecycle POST, and dispatches `sui:edit-mode:change` on document so
 *  shell chrome (which doesn't run inside a tracked effect) can react. */
export function toggleEditMode(name: string): boolean {
  const sig = editModeForView(name)
  const next = !sig()
  setEditMode(name, next)
  return next
}

/** Set edit-mode explicitly. No-op when already at the target value. */
export function setEditMode(name: string, on: boolean): void {
  const sig = editModeForView(name)
  if (sig() === on) return
  sig.set(on)
  void postLifecycle(name, on ? 'enter' : 'exit')
  if (typeof document !== 'undefined') {
    document.dispatchEvent(new CustomEvent('sui:edit-mode:change', {
      detail: { view: name, on },
      bubbles: true,
    }))
  }
}

// ── Annotation mode (sub-toggle inside edit mode) ────────────────────
//
// Annotation mode is a SECOND boolean per (project, view), only
// meaningful when edit-mode is also on. ON ⇒ the overlay binds
// hover/click handlers so the user picks elements to annotate; OFF ⇒
// the view is interactive normally even though the dev panel + tabs
// are still in edit-mode shape.
//
// Session-only: not persisted to localStorage so a reload always
// starts annotation mode off (per the design call D2 — user wants
// each edit session to start clean).

const ANNOTATION_SIGNALS = new Map<string, Signal<boolean>>()

export function annotationModeForView(name: string): Signal<boolean> {
  const k = `ship.annotation-mode.${spaceId()}.${name}`
  const existing = ANNOTATION_SIGNALS.get(k)
  if (existing) return existing
  const sig = signal<boolean>(false)
  ANNOTATION_SIGNALS.set(k, sig)
  return sig
}

export function toggleAnnotationMode(name: string): boolean {
  const sig = annotationModeForView(name)
  const next = !sig()
  setAnnotationMode(name, next)
  return next
}

export function setAnnotationMode(name: string, on: boolean): void {
  const sig = annotationModeForView(name)
  if (sig() === on) return
  // Annotation mode requires edit mode on; refuse to flip on otherwise.
  if (on && !editModeForView(name)()) return
  sig.set(on)
  if (typeof document !== 'undefined') {
    document.dispatchEvent(new CustomEvent('sui:annotation-mode:change', {
      detail: { view: name, on },
      bubbles: true,
    }))
  }
}
