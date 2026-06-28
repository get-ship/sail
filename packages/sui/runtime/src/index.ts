/**
 * @ship/sui-runtime — public API surface.
 *
 * Everything re-exported from this module is part of ship-set's stable
 * runtime contract. Anything not re-exported here is internal and may
 * change without notice.
 *
 * Re-exports keep their original JSDoc — hover any symbol in your
 * editor to see the full docs.
 *
 * Surface map:
 *   - Reactivity:    `signal`, `derived`, `effect`, `batch`, `untrack`
 *                    (+ types `Signal`, `Derived`)
 *   - DOM bindings:  `bindText`, `bindShow`, `bindAttr`, `bindClass`,
 *                    `bindProp`, `bindValue`, `bindChecked`, `bindRawHtml`
 *                    (+ type `Disposer`)
 *   - Events:        `on`, `matchesKey` (+ types `OnOptions`, `Modifier`)
 *   - Control flow:  `bindFor`, `bindForSignal`, `bindIf`, `bindAwait`
 *                    (+ types `BindForOptions`, `RenderedItem`,
 *                    `BindIfOptions`, `BranchRender`, `BindAwaitOptions`)
 *   - Slots:         `resolveSlots`, `snapshotChildren`, `slotFilled`
 *                    (+ type `SlotDecl`)
 *   - Components:    `defineComponent`, `camelOf`
 *                    (+ types `ComponentFactory`, `ComponentSchema`)
 *   - Prop decoders: `decodeString`, `decodeNumber`, `decodeBoolean`,
 *                    `decodeJson`, `decodeEnum` (+ type `Decoder`)
 *   - Data:          `data` (+ types `DataOpts`, `DataHandle`)
 *
 * See .artifacts/specs/ship-set/03-runtime-architecture.md § Public API.
 *
 * @packageDocumentation
 */

// Singleton diagnostic. The whole runtime depends on exactly ONE instance:
// signals, effects, and the context/route registries are module-level state.
// The hard guarantee is structural — the deck bundler externalizes
// `@ship/sui-runtime` for every target so the importmap resolves them all to
// one `/_deck/runtime.js`. This load-time counter is the belt-and-suspenders
// heads-up: a second instance means signals/effects in one copy can't react to
// the other's (split-brain).
//
// In a daemon-served (production) shell this should always be 1 — a 2 there is
// a real bundler-externalization regression. Under the vite shell-dev server it
// can legitimately be 2, because vite serves this module's SOURCE via `@fs`
// alongside the daemon's bundled `/_deck/runtime.js`; that's a dev-server
// artifact, not a bundler bug. Warn (don't throw) so it's visible without
// crying wolf in dev.
{
  const g = globalThis as unknown as { __suiRuntimeInstances?: number }
  g.__suiRuntimeInstances = (g.__suiRuntimeInstances ?? 0) + 1
  if (g.__suiRuntimeInstances > 1) {
    console.warn(
      '[sui-runtime] loaded %d times. In production this means a bundle is not ' +
        'externalizing @ship/sui-runtime (split-brain signals); under vite ' +
        'shell-dev it is the expected @fs-source + /_deck-bundle duplication.',
      g.__suiRuntimeInstances,
    )
  }
}

export { signal, derived, effect, batch, untrack, selector, isSignal } from './signal'
export type { Signal, Derived } from './signal'

// The one MutableState proxy builder. `componentMountState` is emitted by the
// compiler for component-mount; `createMountState` backs both mount paths.
// One read shape: `state.x` is the value (signal keys unwrapped); writes route
// through signal.set via the proxy's set trap.
export { createMountState, componentMountState, pickSignals, SIGNALS_KEY } from './state-proxy'

export {
  createScope,
  createChildScope,
  withScope,
  onCleanup,
  currentScope,
} from './effect-scope'
export type { EffectScope } from './effect-scope'

// Dev-mode leak detector lives at `@ship/sui-runtime/dev` (separate
// subpath import so the wrappers don't ship in the production
// runtime bundle). See packages/sui/runtime/src/dev.ts.

export { shared, isShared, provide, useShared, pickShared } from './context'

// Host→runtime context seam (active project / session). Host-provided,
// like RouterPort/ShipHost — replaces the old window.__ship* globals.
export { setHostContext, hostContext } from './host-context'
export type { HostContext } from './host-context'

export { ref, attachRef } from './ref'
export type { Ref } from './ref'

export { bindSlotEmpty } from './slot-empty'

export {
  bindText,
  bindShow,
  bindAttr,
  bindClass,
  bindProp,
  bindValue,
  bindChecked,
  bindHtml,
} from './bind'
export type { Disposer } from './bind'

export { on, matchesKey } from './on'
export type { OnOptions, Modifier } from './on'

export type { ComponentExample } from './component-example'

export {
  emit,
  expose,
  attachHostApi,
  bindNativeEvent,
  slotted,
  bindHostAttr,
} from './host-api'

export { bindFor, bindForSignal } from './for'
export type { BindForOptions, RenderedItem } from './for'

// Backbone #2 — virtualization — lives on the `@ship/sui-runtime/virtual`
// subpath, NOT this barrel (#297): re-exporting it here dragged
// `@tanstack/virtual-core` (~21KB minified, a third of the bundle) into the
// always-loaded runtime singleton. Windowing consumers import
// `createVirtualizer` / `Virtual` / `VirtualItem` from the subpath; their
// bundles inline the adapter + engine while the reactive core stays the
// external singleton (see src/virtual.ts).

export { bindIf } from './if'
export type { BindIfOptions, BranchRender } from './if'

export {
  registerTransition,
  runIn,
  runOut,
  fade,
  slide,
  scale,
} from './transition'
// runEnter / runLeave / cancelTransitions go through the tiny
// transition-runner indirection (stubs by default). Apps that don't
// use `transition:` directives nor import the named transition
// helpers (`registerTransition`, `fade`, `slide`, `scale`) tree-shake
// `./transition` out entirely — saving ~870 bytes on the Krausest
// bench bundle. When ANY transition export is consumed, transition.ts
// loads + its top-level setTransitionRunner() swaps the stubs for
// the real Web-Animations-API runner.
export { runEnter, runLeave, cancelTransitions } from './transition-runner'
export type {
  Transition,
  TransitionConfig,
  TransitionDir,
  SlideConfig,
  ScaleConfig,
} from './transition'

export { bindAwait } from './await'
export type { BindAwaitOptions } from './await'

export { bindRawHtml } from './raw-html'

export { morph } from './morph'
export type { MorphOptions } from './morph'

export { resolveSlots, snapshotChildren, slotFilled } from './slot'
export type { SlotDecl } from './slot'

export { defineComponent, camelOf } from './component'
export type { ComponentFactory, ComponentSchema } from './component'

export {
  decodeString,
  decodeNumber,
  decodeBoolean,
  decodeJson,
  decodeEnum,
} from './decode'
export type { Decoder } from './decode'

export { data } from './data'
export type { DataOpts, DataHandle } from './data'

export { debounce } from './signal'

export { live, unwrap, unwrapOr } from './live'
export type { LiveState, LiveSignal, LiveSource, LiveOpts, LiveValueOf } from './live'

export { wireFormMutation } from './mutation'

export { bindWhileLoading, bindWhileError } from './while'
export type {
  BindWhileLoadingOptions,
  BindWhileErrorOptions,
} from './while'

// The framework exports only the host seam. The concrete `ShipSDK` + its
// data types live in `@ship/sdk-client` — the runtime names no host shape.
export type { ShipHost, Clients } from './ship'

export { component, prop } from './component'
export type { PropDef, ComponentDef, ComponentSetupContext } from './component'

export type {
  ControllerDef,
  ControllerShape,
  ControllerHook,
  TemplateHook,
  TemplateHookAttrKind,
  TemplateHookContext,
  ViewScope,
  ReadonlyState,
  ReadonlyView,
  MutableState,
  SectionInit,
  OnSection,
  UseSection,
  MountSection,
  UnmountSection,
} from './controller'

export { controller, resolveController, LIFECYCLE_KEY } from './controller-resolver'
export type {
  ControllerExport,
  ControllerLifecycle,
  ResolverContext,
  ResolvedBindings,
} from './controller-resolver'

// The tagless mount-form of `component()` (the function-arg overload) + its
// context/setup/module types. `component` itself is exported above (line 203);
// these are the types the mount-form's setup + default export carry.
export type { MountContext, MountSetup, MountModule } from './component-mount'

// Slice 2 — hydration-marker recording helpers. The synthetic emit calls
// these next to `bindText` / `bindOn` / `bindFor` / etc. so the ship-js
// SSR path emits the marker contract the client `hydrate()` reads. On
// the browser DOM the helpers are no-ops (production walker + ship-js
// SSR already emit markers; CSR doesn't need them).
export {
  markHydrateText,
  markHydrateAttr,
  markHydrateBoundary,
} from './hydrate-mark'

export { useFilteredList, useSelection, useForm } from './hooks'
export type { FilteredListSource, UseFormOptions } from './hooks'

export {
  registerTemplateHook,
  getTemplateHook,
  clearRegistry,
  listRegisteredHooks,
} from './template-hooks/registry'
export type { KbdShortcut } from './template-hooks/kbd'
// Built-in hook factories are loaded on demand via the
// `@ship/sui-runtime/template-hooks` subpath: the SUI compiler emits a
// side-effect import of that subpath into compiled views that contain
// `<use-*>` elements, so hook code only ships with views that use it.
// Register <sui-if>, <sui-for>, <sui-match> custom elements.
import './boundary-elements'

export {
  attachBoundary,
  detachBoundary,
  findNearestBoundary,
  runProtected,
} from './error-boundary'
export type { BoundaryHandle, BoundarySource } from './error-boundary'

export { suiTonesSheet, suiBaseSheet, adoptSuiSheets } from './styles/index'

// F1 mount unification — hosts call mount(target, payload, opts) to put
// a view on screen. See .artifacts/design-f1-mount-unification.md.
export { mount } from './mount'
export type { ViewPayload, MountOptions, MountHandle, SuiLoader } from './mount'

// SUI-local structural route type. Canonical home is the runtime (see
// route-signals.ts) — views read `scope.route.params.<name>()` against this
// shape, and the deck router's `route()` factory returns a structurally
// compatible value. Exported so consumers (and the dom test harness) can
// name `RouteSignals` without importing `@ship/deck-router` (keeps sui ⊥ deck).
export type { RouteSignals } from './route-signals'

// Per-view edit-mode UI state. Used by the dev panel to gate
// view-authoring tabs and annotation rendering. See edit-mode.ts.
export {
  editModeForView, toggleEditMode, setEditMode,
  annotationModeForView, toggleAnnotationMode, setAnnotationMode,
} from './edit-mode'

// Re-export the patch applicator for the same reason as the route
// surface above: `@ship/sdk-client`'s SSE layer imports
// `@ship/sui-runtime/patch`, and the importmap points that subpath at
// the single runtime.js bundle — so the symbols must be reachable here.
export { applyPatch, applyPatches, tokenizePath } from './patch'
export type { Patch, PatchOp, PathToken } from './patch'

// Generated daemon ↔ runtime data contract. Authored by `cargo xtask
// gen-types` from Rust source; do not edit `generated.ts` by hand.
// View-system contract types only — Ship daemon-domain data (git/agents/
// account) lives in @ship/sdk-client, never the framework.
// (LintFinding + the Write* request/response types left this surface when
// the write paths + lint moved to the view-edit Connect wire — their specta
// registrations were retired; see crates/xtask/src/types.rs.)
export type {
  ManifestRail,
  RailMeta,
  ResolveError,
  ResolvedView,
  ViewListEntry,
  ViewManifest,
} from './generated'
