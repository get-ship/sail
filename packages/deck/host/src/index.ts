//! `@ship/deck-host` — Deck's browser-side host.
//!
//! Owns secure route-controller execution (SES Compartment runner). The SAIL
//! runtime provides the hardened endowment surface; the author-time
//! `sail-controller-no-ambient-authority` lint is the static first line; this is
//! the runtime backstop.
//!
//! Ambient authority (`network` / `unconfined`) is resolved from the same
//! `sail.config.json` the compiler walks — pure classify in `./ambient-policy`,
//! Node walk-up in `./ambient-resolve`, live wire-up in
//! `prepareRouteConfinement`. A grant is unsafe-shaped: it voids the default
//! deny at both gates.

export {
  lockdownRealm,
  isRealmLocked,
  confineController,
  createConfinedFetch,
} from './confine'
export {
  classifyAmbient,
  classifyAmbientJson,
  AMBIENT_DENY_ALL,
  SAIL_CONFIG_FILE,
} from './ambient-policy'
export type { AmbientAuthority } from './ambient-policy'
export { findSailConfigPath, resolveAmbientAuthority } from './ambient-resolve'
export { lowerFrameworkImports, lowerModuleToExpression } from './lower-imports'
export {
  buildRouteEndowments,
  confineRouteModule,
  prepareRouteConfinement,
} from './route-runner'
export type {
  RouteEndowmentInput,
  CompiledRouteModule,
  PrepareRouteConfinementInput,
  PreparedRouteModule,
} from './route-runner'
