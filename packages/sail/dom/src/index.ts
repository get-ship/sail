/**
 * @ship/sail-dom — DOM-walk hydrators.
 *
 * F1 mount unification removed `attach()` — hosts call
 * `mount(target, payload)` from `@ship/sail-runtime` instead. ui-dom
 * now exists only to publish the per-kind hydrators the compiler-
 * emitted `hydrate(root, ctx)` calls during SSR resume.
 *
 * The legacy parse-on-the-client subpath has been deleted; every
 * `.sui` file goes through the Rust sail-compiler ahead of time. Tests
 * compile via the wasm-backed `@ship/sail-dom/test-utils` helper.
 *
 * @packageDocumentation
 */

export { loadSetup, pairedTsPath } from './setup-loader'

// Direct-emit hydrators (Phase 3b.2).
//
// The compiler emits per-view/per-component `hydrate(root, ctx)` that
// walks SSR'd DOM via `walk(root, [path])` and dispatches to per-kind
// hydrators below.
export {
  walk,
  hydrateText,
  collectTextOpens,
  collectElementMarkers,
  hydrateAttr,
  hydrateClass,
  hydrateStyle,
  hydrateProp,
  hydrateOn,
  hydrateBindValue,
  hydrateBindChecked,
  hydrateRef,
  hydrateWhileLoading,
  hydrateWhileError,
} from './hydrate'

export { registerCustomProperties } from './register-properties'
