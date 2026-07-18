// theme.ts — the sanctioned runtime theme-write hatch.
//
// A theme write is THE one sanctioned CSS write, sanctioned by SHAPE not by
// permission: it writes DECLARED tokens to `:root` (system-wide). "Applies
// everywhere" is the floor's whole point — the opposite of the off-system
// `el.style`-on-a-random-node write the `no_imperative_style` floor exists to
// kill. This module can ONLY write declared token names to `:root`; an
// arbitrary property, or any target other than `:root`, is refused. That shape
// constraint is the entire sanction — there is NO capability gate and NO
// `sail.config` flag (owner ruling 2026-07-18).
//
// Mechanistically this is the `:root`-targeting sibling of the compiler's
// signal→`(--x)` reflection: that reflection writes a token onto the host via
// the privileged framework path (`root.style.setProperty` inside framework
// code); this writes one to `:root`. Same authority, aimed one level up. The
// `no_imperative_style` lint bans `.style` in controller/pack source precisely
// so this is the ONLY door; framework runtime code (this file) is exempt at the
// dispatch level, so the `root.style.setProperty` below is not a violation.
//
// Surface (see m-deck-standalone/scratch/theme-api-spec.md):
//   theme.preview(patch)  — live :root write (session): the live-preview mechanism
//   theme.commit(patch)   — persist: dev → theme data file (→HMR) via host hook;
//                           live → localStorage (default) or host backend
//   theme.reset()         — drop preview overrides, back to the committed theme
//   theme.current()       — active theme id + resolved token values

/** A patch maps token names (`--foo`) to CSS values. TokenName is validated at
 *  write time against the registered `@property` tokens (or a declared `:root`
 *  var); a non-token key is refused. */
export type ThemeTokenName = string
export type ThemePatch = Record<ThemeTokenName, string>

/** What `theme.current()` reports. */
export interface ThemeState {
  /** Active theme id — the `:root` `data-theme` attribute (empty if unset). */
  id: string
  /** Resolved values of the tokens overridden this session (preview ∪ committed),
   *  read back from `:root` so they reflect what is actually painted. */
  tokens: Record<string, string>
}

/**
 * Host persistence hook. Registered once at host startup via `setThemePersist`.
 *   - dev: serialize the patch into the app's theme data file (the `themes.ts`
 *     fork) so HMR re-themes and it round-trips in source.
 *   - live: write-through to a host-configured backend.
 * Omitted ⇒ persistence is localStorage-only (the default, owner-recommended).
 * Fire-and-forget; a rejected promise is swallowed (the local cache is still
 * correct).
 */
export type ThemePersist = (patch: ThemePatch) => void | Promise<void>

let persistHook: ThemePersist | null = null

/** Register (or clear) the host persistence hook. Call once at host startup.
 *  Same seam shape as `@ship/tokens`' `setThemeTransport` — the runtime names
 *  no daemon URL; the host plugs one in. */
export function setThemePersist(hook: ThemePersist | null): void {
  persistHook = hook
}

const STORAGE_KEY = 'ship-theme-overrides'

// Preview-only overrides applied THIS session (dropped by reset()).
const session = new Map<string, string>()
// Committed overrides — persisted, and NOT dropped by reset().
const committed = new Map<string, string>()

function hasDocument(): boolean {
  return typeof document !== 'undefined'
}

function root(): HTMLElement {
  return document.documentElement
}

// The cross-bundle token registry the compiler's `@property` install writes to
// (see @ship/sail-dom's register-properties.ts). Read it directly off the shared
// global rather than importing @ship/sail-dom — the runtime is the externalized
// singleton and MUST NOT depend on sail-dom (that would invert the layering).
function registeredTokens(): Set<string> {
  return (globalThis as { __sailRegisteredTokens?: Set<string> }).__sailRegisteredTokens ?? new Set()
}

/**
 * A token is writable iff it is a real, DECLARED token:
 *   1. a compiler-`@property`-registered token (the reflection registry), OR
 *   2. a custom property already declared on `:root` (a tokens.css design token,
 *      caught by a non-empty computed value).
 * Anything else — a non-`--` key, or a `--x` nobody declared — is refused. This
 * is the shape sanction: the API cannot invent a token, only rewrite a declared
 * one, and it can only write `:root`.
 */
function isWritableToken(name: unknown): name is string {
  if (typeof name !== 'string' || !name.startsWith('--')) return false
  if (registeredTokens().has(name)) return true
  if (!hasDocument()) return false
  try {
    return getComputedStyle(root()).getPropertyValue(name).trim() !== ''
  } catch {
    return false
  }
}

function refuse(name: string): never {
  throw new TypeError(
    `theme: "${name}" is not a writable token. The theme hatch only writes DECLARED tokens ` +
      `(a compiler-registered \`(--x)\` reflection, or a \`:root\` custom property from tokens.css) ` +
      `to \`:root\`. A non-token key, or any target other than \`:root\`, is refused.`,
  )
}

// Validate the WHOLE patch before touching `:root` — a write is all-or-nothing,
// so a bad key can't leave a half-applied theme behind.
function validated(patch: ThemePatch): Array<[string, string]> {
  const entries = Object.entries(patch)
  for (const [name] of entries) {
    if (!isWritableToken(name)) refuse(name)
  }
  return entries
}

// The privileged `:root` write — the framework path. This is the whole point of
// the module: `root.style.setProperty` here is the same mechanism the compiler's
// reflection runs, aimed at `:root`. Framework runtime code is lint-exempt.
function writeRoot(name: string, value: string): void {
  root().style.setProperty(name, value)
}

/**
 * Live `:root` write for the current session — the live-preview mechanism. Sets
 * each declared token on `:root`; a descendant reading `(--token)` reflects it
 * by inheritance. Session-only: `theme.reset()` drops these; nothing is
 * persisted. Refuses (throws `TypeError`, applies nothing) on a non-token key.
 */
function preview(patch: ThemePatch): void {
  const entries = validated(patch)
  if (!hasDocument()) return
  for (const [name, value] of entries) {
    session.set(name, value)
    writeRoot(name, value)
  }
}

/**
 * Persist a patch as part of the committed theme, and apply it live. In dev the
 * host hook serializes it into the theme data file (→HMR, round-trips in
 * source); live it write-throughs to localStorage (default) or the host backend.
 * A committed token is no longer a session preview, so `theme.reset()` keeps it.
 */
function commit(patch: ThemePatch): void {
  const entries = validated(patch)
  for (const [name, value] of entries) {
    committed.set(name, value)
    session.delete(name) // committed, not a session preview any more
    if (hasDocument()) writeRoot(name, value)
  }
  persist(Object.fromEntries(entries))
}

function persist(patch: ThemePatch): void {
  // Tier-1 local cache (default). The `typeof` guard is not enough — an
  // opaque-origin document throws on the `localStorage` getter itself — so the
  // ACCESS is inside the try (mirrors @ship/tokens' persistTheme).
  try {
    const all = Object.fromEntries(committed)
    localStorage.setItem(STORAGE_KEY, JSON.stringify(all))
  } catch {}
  // Host hook: dev file-write / live backend. Fire-and-forget.
  void persistHook?.(patch)
}

/**
 * Drop this session's preview overrides and fall back to the committed theme.
 * Committed overrides (and the base theme underneath) stay. A token that was
 * only previewed is removed from `:root` (reverting to its committed or base
 * value); a token that is also committed is restored to its committed value.
 */
function reset(): void {
  if (hasDocument()) {
    for (const name of session.keys()) {
      root().style.removeProperty(name)
      const kept = committed.get(name)
      if (kept != null) writeRoot(name, kept) // shouldn't happen (commit clears session) — defensive
    }
  }
  session.clear()
}

/**
 * The active theme id plus the resolved values of every token overridden this
 * session (preview ∪ committed), read back from `:root` so the values reflect
 * what is actually painted.
 */
function current(): ThemeState {
  const names = new Set<string>([...committed.keys(), ...session.keys()])
  const tokens: Record<string, string> = {}
  if (hasDocument()) {
    const cs = getComputedStyle(root())
    for (const name of names) tokens[name] = cs.getPropertyValue(name).trim()
  } else {
    for (const name of names) tokens[name] = committed.get(name) ?? session.get(name) ?? ''
  }
  return {
    id: hasDocument() ? (root().dataset.theme ?? '') : '',
    tokens,
  }
}

/**
 * Wipe ALL runtime theme overrides — both this session's previews and the
 * committed overrides held in memory — and remove them from `:root`. Does NOT
 * touch localStorage or the host backend (a full "forget the persisted theme"
 * is `theme.commit({})` semantics plus a storage clear the host owns). Useful
 * for a host tearing down / switching workspaces, and for test isolation.
 */
export function clearThemeOverrides(): void {
  if (hasDocument()) {
    for (const name of [...session.keys(), ...committed.keys()]) {
      root().style.removeProperty(name)
    }
  }
  session.clear()
  committed.clear()
}

/**
 * Rehydrate committed overrides from localStorage and apply them to `:root`.
 * The live-persistence boot step: a host's pre-paint script calls this so a
 * committed live-preview survives a reload (the dev path needs no boot — the
 * theme data file IS the source, and HMR re-themes). Safe to call server-side
 * (no-op) and idempotent.
 */
export function hydrateTheme(): void {
  if (!hasDocument()) return
  let raw: string | null = null
  try {
    raw = localStorage.getItem(STORAGE_KEY)
  } catch {
    return
  }
  if (!raw) return
  let parsed: unknown
  try {
    parsed = JSON.parse(raw)
  } catch {
    return
  }
  if (parsed == null || typeof parsed !== 'object') return
  for (const [name, value] of Object.entries(parsed as Record<string, unknown>)) {
    if (typeof value !== 'string' || !isWritableToken(name)) continue
    committed.set(name, value)
    writeRoot(name, value)
  }
}

/**
 * `theme` — the sanctioned runtime theme-write hatch. A plain ambient API
 * (`import { theme } from '@ship/sail-runtime'`), NOT `ctx`-scoped: a theme
 * write is a GLOBAL `:root` write, so it needs no view scope.
 */
export const theme = {
  preview,
  commit,
  reset,
  current,
} as const

export type Theme = typeof theme
