// theme.ts — the sanctioned runtime theme-write hatch.
//
// Two write surfaces (both framework-owned, lint-exempt `.style` on purpose):
//
// 1) **`:root` session hatch** (`preview` / `commit` / `reset` / `current`)
//    Writes DECLARED tokens to `:root` only. Shape sanction: non-token keys
//    refuse. Product live-preview / persist — not Studio draft edit.
//
// 2) **Host draft scope** (`applyScope` / `clearScope`)
//    Lowers an in-memory DTCG theme doc (same walk as themes.css emit) and
//    writes `--*` custom properties **only on the given host**. Teardown clears
//    them. Studio edit-loop preview; chrome keeps the app theme on `:root`.
//    Mode comes from args / doc defaultMode — not a second author attr API.
//
// Surface:
//   theme.preview(patch)     — live :root write (session)
//   theme.commit(patch)      — persist + apply on :root
//   theme.reset()            — drop session :root previews
//   theme.current()          — active theme id + resolved override values
//   theme.applyScope(host, { doc, mode }) — host-scoped DTCG draft paint
//   theme.clearScope(host)   — teardown applyScope writes

import registry from '@ship/tokens/token-registry.json'
import type { SailHost } from './sail-host'
import { unwrapSailHost } from './sail-host-dom'

const REGISTRY = registry as {
  schema?: string
  paths?: Record<string, string>
}
if (REGISTRY.schema !== 'spinnaker.registry.v2') {
  throw new Error(
    `runtime theme reads spinnaker.registry.v2 (got ${String(REGISTRY.schema)})`,
  )
}
const REGISTRY_PATHS = REGISTRY.paths ?? {}

function publicNameFromRegistry(path: string): string {
  const mapped = REGISTRY_PATHS[path]
  if (!mapped) {
    throw new Error(`token-registry has no public name for path ${path}`)
  }
  return mapped
}

/** A patch maps token names (`--foo`) to CSS values. TokenName is validated at
 *  write time against the registered `@property` tokens (or a declared `:root`
 *  var); a non-token key is refused. */
export type ThemeTokenName = string
export type ThemePatch = Record<ThemeTokenName, string>

/** What `theme.current()` reports. */
export interface ThemeState {
  /** Active theme id — the `:root` `theme` attribute (empty if unset). */
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
    id: hasDocument() ? (root().getAttribute('theme') ?? '') : '',
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

// ─── applyScope: host-scoped DTCG draft paint ───────────────────────────────
//
// SSOT: THEMES-DRAFT-PAINT-SPEC.md §3.2. Live edit preview paints a **host**
// only (not :root) by lowering the in-memory DTCG doc with the same walk as
// sail-compiler `tokens_generate::flatten_tokens` + `resolve_value`, then
// writing `--cssName` custom properties onto `host`. Descendants inherit.
// Teardown clears only what this API wrote. No themes.css regen, no Studio
// CSSOM.
//
// Mode is an arg (or defaultMode on the doc) — not a second top-level author
// attr. Platform cascade bits set on the host: `theme` + `mode`.
// Never data-theme / data-theme-preview / data-mode.

/** Loose DTCG theme document (modes-in-file or flat). */
export type ThemeDoc = Record<string, unknown>

/** Options for {@link theme.applyScope}. */
export interface ThemeApplyScopeOptions {
  /** In-memory DTCG theme document (project/catalog JSON shape). */
  doc: ThemeDoc
  /**
   * Active paint mode (`light` | `dark` | declared mode id).
   * Omit → `$extensions.sail.defaultMode` → `light` → first `modes` key.
   */
  mode?: string | null
}

/** One lowered token: css custom-property name (`--x`) → CSS value. */
export type ThemeScopePatch = Record<string, string>

/** Leaf from the DTCG walk (mirrors compiler Token before css_value resolve). */
export interface LoweredThemeToken {
  path: string
  cssName: string
  value: string
  cssValue: string
  type: string
}

export interface LowerThemeResult {
  mode: string
  themeId: string
  tokens: LoweredThemeToken[]
  /** `--cssName` → cssValue — ready for host style writes. */
  patch: ThemeScopePatch
}

interface AppliedScope {
  names: string[]
  themeId: string
  mode: string
}

const appliedScopes = new WeakMap<object, AppliedScope>()

function isPlainObject(v: unknown): v is Record<string, unknown> {
  return v != null && typeof v === 'object' && !Array.isArray(v)
}

function isTokenLeaf(node: Record<string, unknown>): boolean {
  return Object.prototype.hasOwnProperty.call(node, '$value')
}

function valueToString(v: unknown): string {
  if (typeof v === 'string') return v
  if (typeof v === 'number' || typeof v === 'boolean') return String(v)
  if (v == null) return 'null'
  // Objects/arrays are not legal DTCG scalar $value for theme emit.
  return ''
}

/**
 * Port of `tokens_generate::flatten_tokens` — walk DTCG groups, collect leaves.
 * Public short names from the generated registry, not model stamps.
 */
function flattenTokens(
  doc: unknown,
  pathPrefix: string[],
  inheritedType: string | undefined,
  out: Array<{ path: string; cssName: string; value: string; type: string }>,
): void {
  if (!isPlainObject(doc)) return
  for (const [key, node] of Object.entries(doc)) {
    if (key.startsWith('$')) continue
    if (!isPlainObject(node)) continue
    if (isTokenLeaf(node)) {
      const pathParts = [...pathPrefix, key]
      const path = pathParts.join('.')
      const cssName = publicNameFromRegistry(path)
      const ty =
        (typeof node.$type === 'string' ? node.$type : undefined) ?? inheritedType ?? 'string'
      const value = valueToString(node.$value)
      if (value === '' && node.$value !== '' && node.$value !== 0 && node.$value !== false) {
        continue
      }
      out.push({ path, cssName, value, type: ty })
    } else {
      const groupType =
        typeof node.$type === 'string' ? node.$type : inheritedType
      flattenTokens(node, [...pathPrefix, key], groupType, out)
    }
  }
}

/** Port of `tokens_generate::parse_alias` — whole-value `{path}` / `{cssName}`. */
function parseAlias(value: string): string | null {
  if (value.length < 3 || value[0] !== '{' || value[value.length - 1] !== '}') return null
  const inner = value.slice(1, -1)
  if (!inner) return null
  for (let i = 0; i < inner.length; i++) {
    const c = inner.charCodeAt(i)
    const ok =
      (c >= 48 && c <= 57) || // 0-9
      (c >= 65 && c <= 90) || // A-Z
      (c >= 97 && c <= 122) || // a-z
      c === 95 || // _
      c === 46 || // .
      c === 45 // -
    if (!ok) return null
  }
  return inner
}

/**
 * Port of `tokens_generate::resolve_value` — `{alias}` → `var(--cssName)`.
 * Index is built from the same document's leaves (path + cssName).
 */
function resolveValue(
  value: string,
  pathToCss: Map<string, string>,
  cssNames: Set<string>,
): string {
  const ref = parseAlias(value)
  if (ref == null) return value
  const cssName = pathToCss.get(ref) ?? (cssNames.has(ref) ? ref : ref.replace(/\./g, '-'))
  return `var(--${cssName})`
}

function themeIdOf(doc: ThemeDoc): string {
  const ext = doc.$extensions
  if (isPlainObject(ext) && isPlainObject(ext.sail) && typeof ext.sail.themeId === 'string') {
    return ext.sail.themeId
  }
  return ''
}

function defaultModeOf(doc: ThemeDoc): string | null {
  const ext = doc.$extensions
  if (
    isPlainObject(ext) &&
    isPlainObject(ext.sail) &&
    typeof ext.sail.defaultMode === 'string' &&
    ext.sail.defaultMode.length > 0
  ) {
    return ext.sail.defaultMode
  }
  return null
}

function modeNodeOf(doc: ThemeDoc, mode: string): { mode: string; node: unknown } {
  const modes = doc.modes
  if (isPlainObject(modes) && Object.keys(modes).length > 0) {
    if (isPlainObject(modes[mode])) return { mode, node: modes[mode] }
    if (isPlainObject(modes.light)) return { mode: 'light', node: modes.light }
    const first = Object.keys(modes)[0]!
    return { mode: first, node: modes[first] }
  }
  // Flat / non-modes draft: walk the whole doc (exclude handled by $ skip).
  return { mode: mode || 'light', node: doc }
}

/**
 * Lower a DTCG theme document for one mode into CSS custom-property writes.
 *
 * Same walk + alias resolve as sail-compiler `tokens_generate` theme layers
 * (see `flatten_tokens` / `resolve_value` in `tokens_generate.rs`). Does **not**
 * call the Rust binary — Studio must paint from RAM draft; this is the TS twin.
 *
 * @returns tokens + `--name` patch; empty tokens if mode branch missing/empty.
 */
export function lowerThemeDoc(
  doc: ThemeDoc,
  mode?: string | null,
): LowerThemeResult {
  const preferred =
    (typeof mode === 'string' && mode.length > 0 ? mode : null) ??
    defaultModeOf(doc) ??
    'light'
  const { mode: resolvedMode, node } = modeNodeOf(doc, preferred)
  const raw: Array<{ path: string; cssName: string; value: string; type: string }> = []
  flattenTokens(node, [], undefined, raw)

  const pathToCss = new Map<string, string>()
  const cssNames = new Set<string>()
  for (const t of raw) {
    pathToCss.set(t.path, t.cssName)
    cssNames.add(t.cssName)
  }

  const tokens: LoweredThemeToken[] = []
  const patch: ThemeScopePatch = {}
  for (const t of raw) {
    const cssValue = resolveValue(t.value, pathToCss, cssNames)
    tokens.push({
      path: t.path,
      cssName: t.cssName,
      value: t.value,
      cssValue,
      type: t.type,
    })
    patch[`--${t.cssName}`] = cssValue
  }

  // Role samples use --display-font / --body-font / --heading-font, which on
  // :root are var() aliases into *-app stacks. applyScope only writes leaves
  // present in the doc — expand concrete stacks onto the public role vars so
  // preview faces actually change when type roles are edited.
  expandFontRoleCascade(patch)

  return {
    mode: resolvedMode,
    themeId: themeIdOf(doc),
    tokens,
    patch,
  }
}

/**
 * Map app font stacks onto the public cascade names sx fontFamily() uses.
 *
 * Theme docs often leave role leaves as `{font-press-start}` / `{font-mono}` while
 * Studio patches concrete stacks onto `*-app` / role leaves. Pure flatten+alias
 * would emit `var(--font-press-start)` without defining that name on the host —
 * the browser then inherits :root pixel stacks. When a concrete stack is present
 * anywhere in the role chain, expand it onto every public name the specimen reads.
 *
 * This is an applyScope live-preview aid. Rust `tokens_generate` does not expand:
 * authors (patchTypeRoles) must write concrete public leaves for Generate parity.
 */
function expandFontRoleCascade(patch: ThemeScopePatch): void {
  /** Prefer concrete stacks; unwrap var(--x) only when --x is already in this patch. */
  const concrete = (key: string, seen: Set<string> = new Set()): string | undefined => {
    if (seen.has(key)) return undefined
    seen.add(key)
    const v = patch[key]
    if (!v) return undefined
    if (!v.startsWith('var(')) return v
    const m = /^var\(\s*--([^),\s]+)/.exec(v)
    if (!m) return undefined
    return concrete(`--${m[1]}`, seen)
  }
  const take = (...keys: string[]): string | undefined => {
    for (const k of keys) {
      const v = concrete(k)
      if (v) return v
    }
    return undefined
  }
  const display = take(
    '--font-display-app',
    '--display-font',
    '--font-display',
    '--heading-font',
    '--typography-heading-font',
  )
  const body = take(
    '--font-body-app',
    '--body-font',
    '--font-body',
    '--typography-body-font',
  )
  const mono = take(
    '--font-mono-app',
    '--mono-font',
    '--font-mono',
    '--typography-mono-font',
  )
  if (display) {
    patch['--font-display-app'] = display
    patch['--font-display'] = display
    patch['--display-font'] = display
    patch['--heading-font'] = display
  }
  if (body) {
    patch['--font-body-app'] = body
    patch['--font-body'] = body
    patch['--body-font'] = body
  }
  if (mono) {
    patch['--font-mono-app'] = mono
    patch['--font-mono'] = mono
    patch['--mono-font'] = mono
  }
}

function writeHostProp(host: HTMLElement, name: string, value: string): void {
  host.style.setProperty(name, value)
}

function clearHostProp(host: HTMLElement, name: string): void {
  host.style.removeProperty(name)
}

/**
 * Apply a DTCG draft onto `host` only (custom properties + theme/mode attrs).
 * Re-entrant: a second call on the same host replaces the previous scope write.
 * Returns a disposer equivalent to {@link clearThemeScope}.
 *
 * Does **not** touch `:root` session preview/commit state.
 * Author surface: pass a {@link SailHost} (opaque); framework unwraps to DOM.
 */
function applyScope(
  host: SailHost,
  options: ThemeApplyScopeOptions,
): () => void {
  const el = host == null ? null : unwrapSailHost(host)
  if (el == null || typeof el.style?.setProperty !== 'function') {
    throw new TypeError('theme.applyScope: host must be a SailHost (web: HTMLElement)')
  }
  if (options == null || !isPlainObject(options.doc)) {
    throw new TypeError('theme.applyScope: options.doc must be a theme document object')
  }

  const lowered = lowerThemeDoc(options.doc, options.mode)
  const prev = appliedScopes.get(el)
  const nextNames = Object.keys(lowered.patch)

  // Drop props from a prior apply that are absent in this patch (mode switch /
  // token delete) so values cannot stick.
  if (prev) {
    const nextSet = new Set(nextNames)
    for (const name of prev.names) {
      if (!nextSet.has(name)) clearHostProp(el, name)
    }
  }

  for (const name of nextNames) {
    writeHostProp(el, name, lowered.patch[name]!)
  }

  // Framework-owned theme IR on the host; mode is a cascade bit (D10/D11).
  // Paint authority attr is `theme` only (matches tokens_generate `[theme=id]`).
  if (lowered.themeId) {
    el.setAttribute('theme', lowered.themeId)
  } else if (prev?.themeId) {
    el.removeAttribute('theme')
  }
  el.setAttribute('mode', lowered.mode)
  // Hard cutover: never dual-write data-theme / data-theme-preview / data-mode.
  if (el.hasAttribute('data-theme')) el.removeAttribute('data-theme')
  if (el.hasAttribute('data-theme-preview')) el.removeAttribute('data-theme-preview')
  if (el.hasAttribute('data-mode')) el.removeAttribute('data-mode')

  appliedScopes.set(el, {
    names: nextNames,
    themeId: lowered.themeId,
    mode: lowered.mode,
  })

  return () => clearThemeScope(host)
}

/**
 * Teardown a prior {@link theme.applyScope} on `host` — removes written custom
 * properties and the theme/mode attrs this API set. Safe if never applied.
 * Author surface: pass a {@link SailHost} (opaque); framework unwraps to DOM.
 */
export function clearThemeScope(host: SailHost): void {
  if (host == null) return
  const el = unwrapSailHost(host)
  const prev = appliedScopes.get(el)
  if (!prev) return
  for (const name of prev.names) {
    clearHostProp(el, name)
  }
  if (prev.themeId) {
    if (el.getAttribute('theme') === prev.themeId) el.removeAttribute('theme')
  }
  if (el.getAttribute('mode') === prev.mode) el.removeAttribute('mode')
  appliedScopes.delete(el)
}

/**
 * `theme` — the sanctioned runtime theme-write hatch.
 *
 * - **`:root` session** (`preview` / `commit` / `reset` / `current`): live
 *   product theme overrides, declared tokens only.
 * - **Host draft scope** (`applyScope`): Studio/in-session preview of a DTCG
 *   document on a nested host — not `:root`, not themes.css.
 *
 * Plain ambient API (`import { theme } from '@ship/sail-runtime'`).
 */
export const theme = {
  preview,
  commit,
  reset,
  current,
  applyScope,
  /** Alias of {@link clearThemeScope} for symmetric teardown. */
  clearScope: clearThemeScope,
} as const

export type Theme = typeof theme
