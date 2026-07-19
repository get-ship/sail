//! `@ship/deck-host` — Deck's browser-side host.
//!
//! Owns secure route-controller execution (SES Compartment runner). The SAIL
//! runtime provides the hardened endowment surface; the author-time
//! `sail-controller-no-ambient-authority` lint is the static first line; this is
//! the runtime backstop.

export { lockdownRealm, isRealmLocked, confineController } from './confine'
export { lowerFrameworkImports, lowerModuleToExpression } from './lower-imports'
export { buildRouteEndowments, confineRouteModule } from './route-runner'
export type { RouteEndowmentInput, CompiledRouteModule } from './route-runner'
