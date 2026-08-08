/**
 * @ship/sail-dom — DOM-walk hydrators.
 *
 * Hosts call `mount(target, payload)` from `@ship/sail-runtime`. This
 * package publishes the per-kind hydrators the compiler-emitted
 * `hydrate(root, ctx)` calls during SSR resume.
 *
 * Every `.sail` file goes through the Rust sail-compiler ahead of time.
 * Tests compile via the wasm-backed `@ship/sail-dom/test-utils` helper.
 *
 * @packageDocumentation
 */

export { loadSetup, pairedTsPath } from './setup-loader'

// Direct-emit hydrators.
//
// The compiler emits per-route/per-component `hydrate(root, ctx)` that
// walks SSR'd DOM via `walk(root, [path])` and dispatches to per-kind
// hydrators below.
export {
  walk,
  hydrateText,
  collectTextOpens,
  collectElementMarkers,
  hydrateAttr,
  hydrateProp,
  hydrateOn,
  hydrateBindValue,
  hydrateBindChecked,
} from './hydrate'

export { registerCustomProperties, isRegisteredThemeToken } from './register-properties'
