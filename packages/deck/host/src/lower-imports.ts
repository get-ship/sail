//! Confined-build transform: lower framework ESM imports to free globals.
//!
//! The SES runner evaluates a controller as source in a Compartment whose
//! framework surface (`signal`, `controller`, `effect`, …) is ENDOWED as
//! globals. The Compartment module loader is proven friction (spike #2), so the
//! confined build instead strips the controller's
//! `import { signal } from '@ship/sail-runtime'` lines — the named bindings then
//! resolve as the free globals the runner endows.
//!
//! Operates on BUILT controller JS (types already erased by tsc/bundler). Only
//! the SAIL framework specifiers are lowered; every other import (a pack, a
//! relative module) is left intact — those are the controller's own deps, not
//! the endowed surface.

/// Strip `import … from '@ship/{sail-runtime|sail-dom|sdk-client}[/subpath]'`
/// (named, default, namespace, or side-effect) so the imported bindings resolve
/// as Compartment globals. Non-framework imports are untouched.
export function lowerFrameworkImports(source: string): string {
  // Matches an import statement whose specifier is a framework module:
  //   import { a, b as c } from '@ship/sail-runtime'
  //   import * as rt from '@ship/sail-dom'
  //   import '@ship/sail-runtime/template-hooks'   (side-effect)
  // The optional `(?:…\s+from\s+)?` clause covers the binding part (which never
  // contains a quote), so side-effect imports (no `from`) also match.
  // `import\s*` (not `\s+`): the compiler emits side-effect imports with no
  // space — `import"@ship/sail-runtime/template-hooks";`. The optional binding
  // clause ends at `\bfrom\s*` so `}from"…"` (minified, no spaces) also matches;
  // a bare side-effect import skips the clause entirely.
  const FRAMEWORK_IMPORT =
    /^[ \t]*import\s*(?:[^'"]*?\bfrom\s*)?['"]@ship\/(?:sail-runtime(?:\/[\w-]+)?|sail-dom|sdk-client)['"][ \t]*;?[ \t]*\r?\n?/gm
  return source.replace(FRAMEWORK_IMPORT, '')
}

/**
 * Lower a built route MODULE to a self-contained expression the SES runner can
 * `Compartment.evaluate()` — which runs source as a script, so ESM `export`
 * syntax must go. We confine the WHOLE compiled route (glue + bundled
 * controller): there's no trust boundary between them, and the glue is just the
 * DOM renderer + runtime wiring, all reachable through endowments (a narrowed
 * node-factory `document`, the runtime surface, the policy-gated ops).
 *
 * Transforms (after {@link lowerFrameworkImports}):
 *   - `export default EXPR`        → `const __sail_default__ = EXPR` (+ a trailing
 *     `__sail_default__;` so evaluate returns it),
 *   - `export const|let|var|function|class …` → strip the `export ` keyword
 *     (the binding stays a module-local),
 *   - `export { … } [from '…']`    → removed.
 *
 * Returns source whose final expression is the module's default export, or the
 * source unchanged (sans framework imports) when there is no default.
 */
export function lowerModuleToExpression(source: string): string {
  let s = lowerFrameworkImports(source)

  // The trailing expression the confined evaluate() must return — the module's
  // default. Two source forms produce it:
  let defaultExpr: string | null = null

  // (a) `export default EXPR` → bind it to a local we can trail.
  s = s.replace(/^([ \t]*)export\s+default\s+/m, (_m, indent: string) => {
    defaultExpr = '__sail_default__'
    return `${indent}const __sail_default__ = `
  })

  // (b) `export { foo as default }` — the form the SAIL compiler actually emits
  // (`export { page_default as default }`, often multi-line). Capture the local
  // bound to `default` BEFORE the export block is stripped below. Only when an
  // `export default` form wasn't already found.
  if (defaultExpr === null) {
    const named = s.match(/export\s*\{[^}]*?\b([A-Za-z_$][\w$]*)\s+as\s+default\b[^}]*\}/)
    if (named) defaultExpr = named[1]!
  }

  // Strip the `export ` keyword from declarations (binding stays module-local).
  s = s.replace(/^([ \t]*)export\s+(?=(?:const|let|var|function|class|async)\b)/gm, '$1')
  // Remove `export { … } [from '…']` lists (multi-line tolerant — `[^}]` spans
  // newlines).
  s = s.replace(/^[ \t]*export\s*\{[^}]*\}\s*(?:from\s*['"][^'"]*['"])?[ \t]*;?[ \t]*\r?\n?/gm, '')

  if (defaultExpr !== null) s += `\n;${defaultExpr};\n`
  return s
}
