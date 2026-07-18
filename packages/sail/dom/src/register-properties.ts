/**
 * `registerCustomProperties` — the unscoped install for compiler-generated
 * `@property` registrations (the `(--x)` binding: a `sx="…"` read that
 * matched a declared prop or a setup-returned signal gets a typed CSS
 * custom-property registration + host reflection — see
 * `sail-compiler`'s `check::sx_vars` / `emit::styles::register_properties_install_js`).
 *
 * Deliberately NOT routed through a component's scoped `styles:` string:
 * that CSS gets wrapped in an `@scope (...) to (...) { ... }` donut at
 * install time (`injectScopedStyles`, in `component-factory.ts`), and
 * `@property` is a top-level-only at-rule — nested inside another at-rule
 * or block, a browser silently drops it. `@property` registrations are also
 * inherently global (the CSSOM only allows one registration per custom-
 * property name, spec-wide), so an unscoped, content-addressed `<style>` in
 * `<head>` is the correct home, not a per-component donut.
 *
 * `id` is content-addressed (an FNV-1a hash of `css`, computed by the
 * compiler) so re-mounting the same component, or two different components
 * that happen to register the same set of properties, both no-op after the
 * first install.
 */
export function registerCustomProperties(id: string, css: string): void {
  // Record the token names this install declares BEFORE the dedupe early-return,
  // so the registry is populated even when the `<style>` is already present
  // (a re-mount, or two components that register the same set). Set adds are
  // idempotent, so this is cheap and safe to run every time.
  recordRegisteredTokens(css)
  if (document.getElementById(id)) return
  const style = document.createElement('style')
  style.id = id
  style.textContent = css
  document.head.appendChild(style)
}

/**
 * The set of custom-property names the compiler has `@property`-registered this
 * session — i.e. the DECLARED tokens the `(--x)` reflection reflects onto a host.
 * Lives on `globalThis` (not a module-level `const`) on purpose: it is the ONE
 * cross-bundle source of truth for "is this a real token", read by the runtime's
 * sanctioned `theme.*` write hatch (`@ship/sail-runtime`'s theme module) to
 * validate a patch. The runtime cannot import this module — that would invert
 * the layering (sail-dom → sail-runtime, never back) — so the seam is a shared
 * global, the same pattern as `__shipHostContext`. Parsing here, reading there,
 * one Set for every bundle that shares the externalized runtime singleton.
 */
declare global {
  // eslint-disable-next-line no-var
  var __sailRegisteredTokens: Set<string> | undefined
}

function registry(): Set<string> {
  return (globalThis.__sailRegisteredTokens ??= new Set<string>())
}

// `@property --foo { ... }` → capture `--foo`. The compiler emits exactly this
// shape (see `emit::styles::register_properties_install_js`); a permissive
// matcher keeps the registry honest even if the block formatting drifts.
const PROPERTY_RULE = /@property\s+(--[\w-]+)/g

function recordRegisteredTokens(css: string): void {
  const reg = registry()
  for (const m of css.matchAll(PROPERTY_RULE)) reg.add(m[1]!)
}

/** True iff `name` is a compiler-registered `@property` token this session.
 *  The runtime's `theme.*` hatch uses this as the first (authoritative) leg of
 *  its TokenName check; a declared-but-unregistered `:root` var is caught by the
 *  runtime's `getComputedStyle` fallback. */
export function isRegisteredThemeToken(name: string): boolean {
  return registry().has(name)
}
