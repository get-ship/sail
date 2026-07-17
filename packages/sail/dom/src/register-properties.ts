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
  if (document.getElementById(id)) return
  const style = document.createElement('style')
  style.id = id
  style.textContent = css
  document.head.appendChild(style)
}
