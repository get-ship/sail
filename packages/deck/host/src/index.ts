//! `@ship/deck-host` — Deck's browser-side host.
//!
//! Owns secure view-controller execution (SES Compartment runner). The SUI
//! runtime provides the hardened endowment surface; the author-time
//! `sui-controller-no-ambient-authority` lint is the static first line; this is
//! the runtime backstop.

export { lockdownRealm, isRealmLocked, confineController } from './confine'
export { lowerFrameworkImports, lowerModuleToExpression } from './lower-imports'
export { buildViewEndowments, confineViewModule } from './view-runner'
export type { ViewEndowmentInput, CompiledViewModule } from './view-runner'
