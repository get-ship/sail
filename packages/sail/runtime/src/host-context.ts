// host-context.ts — the host→runtime context seam.
//
// Some runtime helpers need ambient host facts — which project is active,
// the per-tab session id — to scope their behavior (edit-mode keys,
// view-authoring sessions). Historically the host stashed these on
// `window.__shipProjectId` / `__shipSessionId` and the runtime fished them
// off `window`. That's the same leak the router had: host state on a global
// instead of a declared seam (and in practice nothing primed them, so the
// reads were dead).
//
// `HostContext` replaces that: the host registers it once via
// `setHostContext`, the runtime reads it via `hostContext()`. Same pattern
// as `RouterPort` / `ShipHost` / `SuiLoader` — the runtime names no host
// concept, the host plugs one in. Held on `globalThis` so it's visible to
// every bundle sharing the single `@ship/sail-runtime` instance.

/** Ambient host facts the runtime may read. All optional — a host that
 *  doesn't know a value omits it, and the runtime falls back. */
export interface HostContext {
  /** Active project/workspace id. Scopes per-project runtime state. */
  spaceId?: string
  /** Per-tab session id, shared with the view-authoring handshake. */
  sessionId?: string
  /** Edit-mode lifecycle transport. The host wires this to its view
   *  service enter/exit RPC; the runtime calls it on toggle and names no
   *  daemon URL. Omitted ⇒ the lifecycle notify is a no-op (the local
   *  toggle still works). */
  editModeTransport?: (name: string, action: 'enter' | 'exit') => void | Promise<void>
}

/** Register the host context. Call at host startup and whenever a value
 *  changes (e.g. project switch). Merges over any previously-set values. */
export function setHostContext(ctx: HostContext): void {
  const prev = globalThis.__shipHostContext ?? {}
  globalThis.__shipHostContext = { ...prev, ...ctx }
}

/** Read the current host context. Empty object when no host registered
 *  one (a non-shell host, or tests) — callers supply their own fallback. */
export function hostContext(): HostContext {
  return globalThis.__shipHostContext ?? {}
}

declare global {
  // eslint-disable-next-line no-var
  var __shipHostContext: HostContext | undefined
}
