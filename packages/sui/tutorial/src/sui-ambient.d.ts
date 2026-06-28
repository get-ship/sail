// Ambient module declarations for the tutorial's `.sui` templates.
//
// WHY THIS EXISTS: each lesson's `controller.ts` does
//   `import type { Scope } from './template.sui'`
// where `template.sui` is a SUI template file (a custom file extension that
// the sui-compiler would normally emit an adjacent `<stem>.sui.d.ts` for —
// see `scripts/gen-component-types.mjs` and the component packs under
// `packages/components/sui/*`). The tutorial is a docs asset, not a
// component pack, and the compiler is a Rust binary that isn't required
// for `typecheck` here, so we resolve the import via a wildcard ambient
// declaration instead.
//
// `Scope` is the per-template binding shape that controllers parameterize
// their `controller<Scope>(…)` and `MutableState<Scope>` calls on. The
// structural shape the runtime consumes is `Record<string, unknown>` —
// `MutableState<Scope>` is built off `StateKeys<Scope>` (a mapped type
// over signal-valued keys) plus the non-signal scope keys; the tests
// themselves don't reference `Scope` properties directly, they pass it
// straight through to the controller factory. `Record<string, unknown>`
// is therefore a precise-enough stand-in without pretending the compiler
// generated per-binding types we don't have.

declare module '*.sui' {
  // eslint-disable-next-line @typescript-eslint/consistent-type-definitions
  export type Scope = Record<string, unknown>;
}
