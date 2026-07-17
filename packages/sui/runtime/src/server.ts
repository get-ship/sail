// @ship/sui-runtime/server — the SERVER (V8 / no-browser) entry.
//
// The synthetic SSR build aliases `@ship/sui-runtime` to THIS module. It
// re-exports the environment-agnostic reactive core + the DOM bindings from
// their source modules, deliberately bypassing the package index
// (`./index.ts`) for two reasons:
//
//   1. The index runs a browser-only module side effect on load —
//      `import './boundary-elements'` calls `customElements.define(...)`,
//      and there is no `customElements` in a V8 isolate. (The call is
//      guarded by `typeof customElements`, so it is inert; the point is to
//      not pull the whole browser graph in at all.)
//   2. The index transitively imports `./generated` (Specta/proto codegen
//      types) and the full view/controller/mount/chrome graph — none of
//      which a synthesized render or controller needs.
//
// The bindings (`bindText`, `bindAttr`, `bindIf`, `bindFor`, …) only touch
// the DOM *when called*, through the `node`/`el` they're handed. Server-side
// the SSR entry installs a minimal server-DOM as `globalThis.document`, so
// the SAME binder runs unchanged and serializes to HTML — there is one
// isomorphic render artifact, not a second string-renderer emit path.

// ── reactive core (pure; no DOM) ────────────────────────────────────────────
export {
  signal,
  derived,
  effect,
  batch,
  untrack,
  selector,
  isSignal,
  debounce,
  type Signal,
} from './signal'
export { live, unwrap, unwrapOr, type LiveSource, type LiveSignal, type LiveState } from './live'
export { data } from './data'
export { createMountState, componentMountState, pickSignals, SIGNALS_KEY } from './state-proxy'
export { ref, attachRef } from './ref'

// ── scope + store (scope-homed shared state) ────────────────────────────────
export {
  createScope,
  createChildScope,
  withScope,
  onCleanup,
  currentScope,
  provideContext,
  injectContext,
  contextKey,
  walkScope,
  SuiMountOutsideSetupError,
} from './effect-scope'
export type { EffectScope, ContextKey } from './effect-scope'
export { mount as mountAmbient } from './effect-scope'
export {
  defineStore,
  useStore,
  provideStore,
  peekStore,
  setStoreSSRPayload,
  clearStoreSSRPayload,
  readStoreSSRSeed,
  snapshotStore,
  snapshotScopeStores,
  SSR_PAYLOAD_GLOBAL,
} from './store'
export type { StoreDef, StoreInstance } from './store'

// ── controller DSL (pure; no DOM) ───────────────────────────────────────────
// The synthetic controller composes an authored `controller({ scope, derived,
// on })` by calling its `setup(ctx)` — the same resolver the browser hydrate
// path runs. `controller`/`resolveController` are environment-agnostic (they
// build signal/derived/on bindings, no DOM), so they belong in the server
// entry too: a synthesized view's authored controller imports `controller`
// from `@ship/sui-runtime`, which the SSR build aliases here.
export { controller, resolveController } from './controller-resolver'
export type {
  ControllerExport,
  ResolverContext,
  ResolvedBindings,
} from './controller-resolver'

// ── DOM bindings (operate on the node they're given — server-DOM or real) ────
export {
  bindText,
  bindShow,
  bindAttr,
  bindClass,
  bindProp,
  bindValue,
  bindChecked,
  bindHtml,
  type Disposer,
} from './bind'
export { on, matchesKey } from './on'
export { wireFormMutation } from './mutation'
// Slice 2 — the synthetic emit stamps `markHydrateText(...)` next to every
// `bindText` / `bindFor` / `bindIf` dynamic-text position so ship-js SSR
// emits the `<!--sui:text:bN-->` markers the client `hydrate()` reads.
// The helpers are no-ops on the browser DOM and the SSR server-DOM stamps
// the field the server-DOM serialization reads. Re-exported from the
// server entry so synthesized renders type-check + resolve when the SSR
// build aliases `@ship/sui-runtime` here.
export { markHydrateText, markHydrateAttr, markHydrateBoundary } from './hydrate-mark'
// ── control flow ────────────────────────────────────────────────────────────
export { bindIf } from './if'
export { bindFor, bindForSignal } from './for'
export { bindWhileLoading, bindWhileError } from './while'

// ── the server DOM (SSR-only; one isomorphic render artifact) ────────────────
// `installServerDom()` is what makes the SAME render + bindings above run in a
// V8 isolate (no browser) and serialize to first-paint HTML. The SSR entry
// installs it before mounting; see `server-dom.ts`.
export { installServerDom } from './server-dom'
export type { ServerRoot } from './server-dom'

// ── slots / boundaries / transitions / hooks ────────────────────────────────
export { bindSlotEmpty } from './slot-empty'
export { resolveSlots, snapshotChildren, slotFilled } from './slot'
export { getTemplateHook, registerTemplateHook } from './template-hooks/registry'
export { adoptSuiSheets } from './styles/index'
