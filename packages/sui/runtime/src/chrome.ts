// chrome.ts — typed IPC wrapper for shell-owned chrome surfaces.
//
// Replaces the ad-hoc `dispatchEvent(new CustomEvent('sui:notify'…))`
// pattern with a single typed API. Every call lands on `document` as a
// CustomEvent; shell owns the listeners (see apps/shell/src/chrome/
// registry.ts). Old event names still fire — chrome continues to listen
// to the raw shapes identically, so legacy call sites keep working
// during migration.
//
// Slot taxonomy
//   - notify       — fire-and-forget toast
//   - navigate     — URL change request
//   - dock.*       — bottom-panel orchestration
//   - bar.*        — command-bar visibility
//   - breadcrumb   — per-view (last set wins)
//   - status       — per-key map (keyed claims, auto-clear on unmount)
//   - theme        — per-token CSS custom property overrides
//   - focus        — boolean stack push/pop (any push hides rail+status)
//   - modal.request — Promise that resolves with the chosen action label
//
// Auto-clear pattern
//   State-bearing slots (breadcrumb, status, theme, focus) stamp each
//   call with the caller's view id. Shell2 maintains a `viewId → claims`
//   registry and clears claims when the mount handle disposes. The view
//   id is read from `window.__suiCurrentViewId`, which shell updates as
//   the mounted route changes. Callers can override with `{ viewId }` on
//   any state-bearing call.

// Slice 2 — `navigate` no longer imports from `@ship/deck/router`. The
// runtime's own `navigate()` (exported from `./mount` / wired through
// the host's `RouterPort`) is the canonical entry; chrome calls that
// via the global `__suiNavigate` set by the host. Falls back to a
// CustomEvent `sui:navigate` dispatch for legacy hosts — the unified
// router listens to both. (Chrome is slated to die; this fallback
// is the minimum decouple, not a new feature.)

// Host-set global navigate hook. Declared as a local cast helper instead
// of `declare const globalThis: … & typeof globalThis` (which is self-
// referential — TS2502) or a `declare global` augmentation (which would
// leak `__suiNavigate` onto every consumer's global scope).
type SuiNavigateGlobal = { __suiNavigate?: (path: string, opts?: { replace?: boolean }) => void }

/** Toast / status tone palette. Matches the values `<sui-toast>` paints
 *  (`info` · `success` · `warning` · `destructive`). `error` is accepted
 *  as an alias for `destructive` so callers can speak the more natural
 *  noun — the wrapper normalizes on dispatch. */
export type NotifyTone = 'info' | 'success' | 'warning' | 'destructive' | 'error'

export interface NotifyOpts {
  tone?: NotifyTone
  title?: string
  description?: string
  duration?: number
  dismissible?: boolean
  showProgress?: boolean
  actionHtml?: string
}

export interface BreadcrumbItem {
  label: string
  href?: string
}

export interface StatusValue {
  label: string
  tone?: NotifyTone
  icon?: string
}

export interface NavigateOpts {
  replace?: boolean
}

export interface ScopeOpt {
  /** View-id stamp for auto-clear. Defaults to the shell's current view. */
  viewId?: string
}

export type ModalAction =
  | string
  | {
      label: string
      tone?: 'default' | 'primary' | 'destructive'
      /** When set, this action triggers if user presses ESC / backdrop. */
      cancel?: boolean
    }

export interface ModalRequestOpts {
  title: string
  description?: string
  /** Action labels in display order. The Promise resolves with the label
   *  the user selected, or with the `cancel`-flagged label on dismiss. */
  actions: ModalAction[]
  dismissible?: boolean
}

interface ChromeWindow extends Window {
  __suiCurrentViewId?: string
}

function getCurrentViewId(): string | undefined {
  if (typeof window === 'undefined') return undefined
  return (window as ChromeWindow).__suiCurrentViewId
}

function emit(name: string, detail: unknown): void {
  if (typeof document === 'undefined') return
  document.dispatchEvent(new CustomEvent(name, { detail, bubbles: true }))
}

function stamp<T extends object>(detail: T, opts?: ScopeOpt): T & { viewId?: string } {
  const viewId = opts?.viewId ?? getCurrentViewId()
  return viewId ? { ...detail, viewId } : detail
}

// ── Modal response wiring ───────────────────────────────────────────
//
// `chrome.modal.request` returns a Promise that resolves when shell's
// modal renderer dispatches `sui:modal:respond` with the matching id.
// The map is module-local so request ids stay unique within a runtime.

let modalSeq = 0
const pendingModals = new Map<number, (action: string) => void>()

// Pre-fix: the `sui:modal:respond` listener attached at module-top-
// level on import — a side effect that pinned chrome.ts into every
// bundle that pulled the runtime barrel, even if no consumer used
// `chrome.modal.request()`. Listener attachment moves to the first
// request() call below so the top-level becomes side-effect-free
// and esbuild can drop chrome.ts when `chrome` isn't imported.
let modalListenerAttached = false
function attachModalListener(): void {
  if (modalListenerAttached) return
  if (typeof document === 'undefined') return
  modalListenerAttached = true
  document.addEventListener('sui:modal:respond', (e: Event) => {
    const detail = (e as CustomEvent).detail as
      | { id?: number; action?: string }
      | undefined
    if (typeof detail?.id !== 'number') return
    const resolve = pendingModals.get(detail.id)
    if (!resolve) return
    pendingModals.delete(detail.id)
    resolve(detail.action ?? '')
  })
}

// ── Public API ──────────────────────────────────────────────────────

function normalizeTone(tone: NotifyTone | undefined): Exclude<NotifyTone, 'error'> | undefined {
  return tone === 'error' ? 'destructive' : tone
}

export const chrome = {
  notify(opts: NotifyOpts): void {
    const normalized = normalizeTone(opts.tone)
    const detail: Record<string, unknown> = { ...opts }
    if (normalized !== undefined) detail['tone'] = normalized
    else delete detail['tone']
    emit('sui:notify', detail)
  },
  navigate(path: string, opts?: NavigateOpts): void {
    // Prefer the runtime's host-wired `navigate` (set by `mount.ts` from
    // the host's `RouterPort`). Fall back to a CustomEvent dispatch for
    // legacy hosts that haven't wired the unified router.
    const replace = opts?.replace ? { replace: true } : undefined
    const suiNavigate = (globalThis as SuiNavigateGlobal).__suiNavigate
    if (typeof suiNavigate === 'function') {
      suiNavigate(path, replace)
    } else {
      emit('sui:navigate', { path, replace: !!opts?.replace })
    }
  },

  dock: {
    toggle(): void { emit('sui:dock:toggle', {}) },
    show(): void { emit('sui:dock:show', {}) },
    hide(): void { emit('sui:dock:hide', {}) },
    activate(tab: string): void { emit('sui:dock:activate', { tab }) },
    clear(tab?: string): void { emit('sui:dock:clear', tab ? { tab } : {}) },
    popout(tab?: string): void { emit('sui:dock:popout', tab ? { tab } : {}) },
    close(): void { emit('sui:dock:close', {}) },
    resize(height: number): void { emit('sui:dock:resize', { height }) },
  },

  bar: {
    open(mode?: string, prefill?: string): void {
      const detail: Record<string, unknown> = {}
      if (mode !== undefined) detail['mode'] = mode
      if (prefill !== undefined) detail['prefill'] = prefill
      emit('ship:bar:open', detail)
    },
    close(): void { emit('ship:bar:close', {}) },
  },

  breadcrumb: {
    set(items: BreadcrumbItem[], opts?: ScopeOpt): void {
      emit('sui:breadcrumb:set', stamp({ items }, opts))
    },
    clear(opts?: ScopeOpt): void {
      emit('sui:breadcrumb:set', stamp({ items: [] as BreadcrumbItem[] }, opts))
    },
  },

  status: {
    set(key: string, value: StatusValue, opts?: ScopeOpt): void {
      emit('sui:status:set', stamp({ key, value }, opts))
    },
    clear(key: string, opts?: ScopeOpt): void {
      emit('sui:status:clear', stamp({ key }, opts))
    },
  },

  theme: {
    set(token: string, value: string, opts?: ScopeOpt): void {
      emit('sui:theme:set', stamp({ token, value }, opts))
    },
    clear(token: string, opts?: ScopeOpt): void {
      emit('sui:theme:clear', stamp({ token }, opts))
    },
  },

  focus: {
    enter(opts?: ScopeOpt): void {
      emit('sui:focus:enter', stamp({}, opts))
    },
    exit(opts?: ScopeOpt): void {
      emit('sui:focus:exit', stamp({}, opts))
    },
  },

  modal: {
    request(opts: ModalRequestOpts): Promise<string> {
      attachModalListener()
      const id = ++modalSeq
      return new Promise((resolve) => {
        pendingModals.set(id, resolve)
        emit('sui:modal:request', { id, ...opts })
      })
    },
  },
} as const

/** Shell-side helper: announces that a view unmounted so the chrome
 *  registry can release per-view claims (breadcrumb, status keys, theme
 *  tokens, focus push). Not for view authors — shell's mountRoute calls
 *  this from its dispose path. */
export function notifyViewUnmount(viewId: string): void {
  emit('sui:view:unmount', { viewId })
}
