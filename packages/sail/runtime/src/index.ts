/**
 * @ship/sail-runtime — public API surface.
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
 *   - Control flow:  `bindFor`, `bindForSignal`, `bindIf`
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
// `@ship/sail-runtime` for every target so the importmap resolves them all to
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
      '[sail-runtime] loaded %d times. In production this means a bundle is not ' +
        'externalizing @ship/sail-runtime (split-brain signals); under vite ' +
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
  provideContext,
  injectContext,
  contextKey,
  walkScope,
  SuiMountOutsideSetupError,
} from './effect-scope'
export type { EffectScope, ContextKey } from './effect-scope'
// Ambient controller/component lifecycle `mount(fn)` — aliased on export
// because `mount` (below) is already the host-facing "mount a route into
// the DOM" entry point (F1). The compiler's ambient-injection table maps
// the authored bare identifier `mount` to this export under that alias
// (`crates/sail/compiler/src/runtime_ambient.rs`), so a controller body
// can write `mount(fn)` with no import while `@ship/sail-runtime`'s own
// `mount(target, payload, opts)` keeps its name for hosts.
export { mount as mountAmbient } from './effect-scope'

// Store API — scope-homed shared state. `defineStore(setup)` is
// the registration; `useStore(def)` resolves the instance from
// the current scope's parent chain (root-or-nearest). The SSR
// payload handshake (`setStoreSSRPayload` server-side,
// `hydrateStoreSSRPayload` client-side) hands the per-request
// signal values from server to client before the first effect
// runs — no flash, no double-init.
export {
  defineStore,
  useStore,
  provideStore,
  peekStore,
  setStoreSSRPayload,
  clearStoreSSRPayload,
  readStoreSSRSeed,
  hydrateStoreSSRPayload,
  snapshotStore,
  snapshotScopeStores,
  SSR_PAYLOAD_GLOBAL,
} from './store'
export type { StoreDef, StoreInstance } from './store'

// Host→runtime context seam (active project / session). Host-provided,
// like RouterPort/ShipHost — replaces the old window.__ship* globals.
export { setHostContext, hostContext } from './host-context'
export type { HostContext } from './host-context'

// The sanctioned runtime theme-write hatch — `theme.preview/commit/reset/
// current`. A plain ambient API (NOT ctx-scoped): a theme write is a GLOBAL
// `:root` write of DECLARED tokens, the `:root`-targeting sibling of the
// compiler's signal→`(--x)` reflection. `setThemePersist` is the host
// persistence seam (dev file-write / live backend); `hydrateTheme` is the
// optional live-persistence boot step. See theme.ts.
export { theme, setThemePersist, hydrateTheme, clearThemeOverrides } from './theme'
export type { Theme, ThemePatch, ThemeTokenName, ThemeState, ThemePersist } from './theme'

export { bindSlotEmpty } from './slot-empty'

export {
  bindText,
  bindShow,
  bindAttr,
  bindClass,
  bindProp,
  bindDynamic,
  bindValue,
  bindChecked,
  bindHtml,
} from './bind'
export type { Disposer } from './bind'

export { on, matchesKey } from './on'
export type { OnOptions, Modifier } from './on'

// The `use:` directive runtime — attaches a reusable "action" (element →
// behavior + teardown) to an element. The compiler emits `runAction` for
// `use:action` / `use:action={param}`. The primitive that removes
// author-facing ref + effect for element behavior.
// Compositions — plain functions over the primitives. Part of the LANGUAGE, not a
// package you install: `form()` is no more optional than `signal()`.
export { form, selection, filteredList } from './compositions'
export type { FormOptions, ListSource } from './compositions'

export { runAction } from './action'
export type { Action, ActionResult } from './action'

// Built-in `use:` actions — reusable element behaviors (autosize, clickOutside…)
// the L1 components lean on. `use:<name>` attaches them; no hand-wired refs/effects.
// Every built-in `use:` action. The compiler AUTO-IMPORTS these by name when a
// template uses one (see `BUILTIN_ACTIONS` in lower/bindings.rs), so an action
// missing here is not a missing convenience — it is an L1 element that compiles,
// passes conformance, and then fails to bundle. `builtin_actions_are_exported`
// in the compiler's test suite holds the two lists together.
export {
  alert,
  anchor,
  autosize,
  avatar,
  chip,
  chromeOffset,
  clickOutside,
  codeBlock,
  collapsible,
  combobox,
  contextMenu,
  dropzone,
  emptyState,
  errorBoundary,
  errorState,
  field,
  hoverCard,
  menu,
  pagination,
  radioGroup,
  rangeFill,
  tabs,
  icon,
  primeIconCache,
  themePicker,
  toast,
  toggleGroup,
  tooltip,
  virtualRow,
  virtualSpacer,
  cellPad,
  rect,
  pin,
  portEdge,
  eventSpan,
  worldTransform,
  zoomPan,
  maxWidth,
  swatch,
  spotlight,
  targetCursor,
  gridTracks,
} from './actions'
export type { AnchorPlacement, PortSide } from './actions'

// Scroll-lock / focus-return — runtime seams for overlays (dialog, drawer).
// The body is part of the DOCUMENT, not a component, so a per-element `use:`
// action cannot reach it. Controllers import these directly; the framework
// layer (packages/sail/runtime/src/**) exempts them from the imperative-style
// lint — the body lock IS the seam.
export {
  focusableWithin,
  FocusReturn,
  acquireScrollLock,
  releaseScrollLock,
} from './scroll-lock'

// Native <dialog> behavior — the compiler emits bindOpen for
// `<dialog bind:open={state}>`. Thin sugar over showModal()/close(); the
// platform keeps the focus trap, top layer, ESC, and focus-return.
export { bindOpen } from './enhance-dialog'
export type { DialogOptions } from './enhance-dialog'

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

// Backbone #2 — virtualization — lives on the `@ship/sail-runtime/virtual`
// subpath, NOT this barrel (#297): re-exporting it here dragged
// `@tanstack/virtual-core` (~21KB minified, a third of the bundle) into the
// always-loaded runtime singleton. Windowing consumers import
// `createVirtualizer` / `Virtual` / `VirtualItem` from the subpath; their
// bundles inline the adapter + engine while the reactive core stays the
// external singleton (see src/virtual.ts).

export { bindIf } from './if'
export type { BindIfOptions, BranchRender } from './if'

// runEnter / runLeave / cancelTransitions go through the tiny
// transition-runner indirection (stubs by default). Apps that don't
// use `transition:` directives nor import the named transition
// helpers (`registerTransition`, `fade`, `slide`, `scale`) tree-shake
// `./transition` out entirely — saving ~870 bytes on the Krausest
// bench bundle. When ANY transition export is consumed, transition.ts
// loads + its top-level setTransitionRunner() swaps the stubs for
// the real Web-Animations-API runner.
// The `transition:` catalog (registerTransition / fade / slide / scale) is GONE —
// it was a second animation system beside `motion:`, with its own presets, its own
// registry and its own runtime. One system now: `motion:enter` / `motion:exit` bake
// to `el.animate()` from the same spring catalog as every other motion trigger.
//
// The lifecycle SEAM survives, because the capability is real: `if.ts` / `for.ts`
// await `runLeave` before removing nodes, which is how an element stays in the DOM
// long enough to animate out — the one thing CSS cannot do alone. It is backed by
// motion.ts now (installed lazily, the first time a `motion:exit` attaches), so an
// app with no exit animations still tree-shakes the whole thing away.
// Template cloning — the compiler hoists one `__tpl(html)` per static subtree.
export { __tpl } from './tpl'
// Event delegation — one root listener per event type; handlers live on the node.
export { delegateEvents } from './delegate'
export { runEnter, runLeave, cancelTransitions, setTransitionRunner } from './transition-runner'

export { bindRawHtml } from './raw-html'

export { morph } from './morph'
export type { MorphOptions } from './morph'

export { resolveSlots, snapshotChildren, slotFilled } from './slot'
export type { SlotDecl } from './slot'

export { defineComponent, camelOf } from './component'
// Element-scope tree (browser side): the per-document root element
// scopes chain to, plus the read-side stamp lookup for diagnostics.
export { clientRootScope, elementScopeOf } from './element-scope'
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

// query()/mutation() — the read/write primitives. `live()` is DELETED
// (data-arch-seed.md, "Owner RULINGS 2026-07-18"): its PULL half is
// query() (Promise fetcher) and its PUSH half folded into query() by
// type-directed routing at the runtime seam (LiveSource fetcher). The
// loading/ok/error + push-source types, and `unwrap`/`unwrapOr`, are
// owned by query.ts now.
export { query, queryKeyed, mutation, __invalidate, unwrap, unwrapOr } from './query'
export type {
  QueryOpts,
  QueryKey,
  MutationOpts,
  MutationState,
  Mutation,
  LiveState,
  LiveSignal,
  LiveSource,
  LiveValueOf,
} from './query'

export { wireFormMutation } from './mutation'

export { bindWhileLoading, bindWhileError } from './while'
export type {
  BindWhileLoadingOptions,
  BindWhileErrorOptions,
} from './while'

// The framework exports only the host seam. The concrete `ShipSDK` + its
// data types live in `@ship/sdk-client` — the runtime names no host shape.
// `ShipHost` types the MOUNT seam only (see ./ship); `Clients` is the
// SAIL-local data port the controller scope is typed against (see ./clients).
export type { ShipHost } from './ship'
export type { Clients } from './clients'

export { component, prop } from './component'
// Child composition: mount an (internal, unregistered) component module into a
// host node within a parent's render. The compiler emits a call to this for an
// underscore-named component reference; registration stays the export path.
export { mountComponent } from './compose'
export type { PropDef, ComponentDef, ComponentSetupContext } from './component'

export type {
  ControllerDef,
  ControllerSetup,
  TemplateHook,
  TemplateHookAttrKind,
  TemplateHookContext,
  ControllerScope,
  ReadonlyState,
  ReadonlyView,
  MutableState,
} from './controller'

export { controller, resolveController } from './controller-resolver'
export type {
  ControllerExport,
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


export { bindPopover } from './enhance-popover'
export type { PopoverOptions } from './enhance-popover'

export {
  registerTemplateHook,
  getTemplateHook,
  clearRegistry,
  listRegisteredHooks,
} from './template-hooks/registry'
export type { KbdShortcut } from './template-hooks/kbd'
// Built-in hook factories are loaded on demand via the
// `@ship/sail-runtime/template-hooks` subpath: the SAIL compiler emits a
// side-effect import of that subpath into compiled routes that contain
// `<use-*>` elements, so hook code only ships with routes that use it.
// Register <sail-if>, <sail-for>, <sail-match> custom elements.
import './boundary-elements'

export {
  attachBoundary,
  detachBoundary,
  findNearestBoundary,
  runProtected,
} from './error-boundary'
export type { BoundaryHandle, BoundarySource } from './error-boundary'

// `motion:` directive runtime — AOT-baked `el.animate(...)` + a tiny
// attach helper that wires the animations to a DOM event per
// trigger. Apps that don't use `motion:` directives don't import
// `__sui_motion_attach` from the emitted code, so this export is
// tree-shaken when unused.
export { __sui_motion_attach } from './motion'
export type { MotionAttachConfig, MotionConfig } from './motion'

// The choreography lane. `motion:` (above) owns ambient, fire-and-forget
// animation and hands it to the compositor; `timeline()` owns motion whose
// clock something ELSE holds — a scrub, a scroll, a sequence of stops that
// must land on exact offsets. Keyframes go in as data, signals come out, and
// the existing `(--name)` reflection carries them to the style layer.
export { timeline, SailTimelineError } from './timeline'
export type {
  TimelineValue,
  TimelineEasing,
  TimelineEasingName,
  TimelineStop,
  TimelineStopOptions,
  TimelineTrack,
  TimelineTracks,
  TimelineSignals,
  TimelineOptions,
} from './timeline'

// Stock progress drivers. Each returns a plain signal, so a hand-written
// `signal(0)` a slider writes is interchangeable with any of them.
export { clock, scrollProgress, springValue } from './drivers'
export type { ClockOptions, ClockSignal, ScrollProgressOptions, SpringOptions } from './drivers'
// Shared-sheet adoption for shadow components. Compiler-emitted client
// code calls `adoptSuiSheets(__root)` in every component mount (see
// crates/sail/compiler/src/emit/js/setup.rs). It is client-safe (no-ops
// without ShadowRoot) and MUST be on the client entry, not just server.ts,
// or the bundled runtime lacks the export the emit imports.
export { adoptSuiSheets } from './styles/index'
// F1 mount unification — hosts call mount(target, payload, opts) to put
// a route on screen. See .artifacts/design-f1-mount-unification.md.
export { mount } from './mount'
export type { RoutePayload, MountOptions, MountHandle, SuiLoader } from './mount'

// SAIL-local structural route type. Canonical home is the runtime (see
// route-signals.ts) — routes read `scope.route.params.<name>()` against this
// shape, and the deck router's `route()` factory returns a structurally
// compatible value. Exported so consumers (and the dom test harness) can
// name `RouteSignals` without importing `@ship/deck-router` (keeps sail ⊥ deck).
export type { RouteSignals } from './route-signals'

// Re-export the route factory (the passive RouteSignals reader) for the SAME
// reason as the patch surface below: a compiled route imports `route` from
// `@ship/sail-runtime/route`, and the deck importmap points that subpath at the
// single `runtime.js` bundle — so the symbol must be reachable from THIS entry
// or production resolves it to a bundle that doesn't export it. (Policy —
// history/navigate/route-table — stays in the deck router; this is the SAIL
// mechanism half: sail ⊥ deck.)
export { route, routeWithUpdate } from './route'
export type { RouteHandle } from './route'

// Per-route edit-mode UI state. Used by the dev panel to gate
// route-authoring tabs and annotation rendering. See edit-mode.ts.
export {
  editModeForRoute, toggleEditMode, setEditMode,
  annotationModeForRoute, toggleAnnotationMode, setAnnotationMode,
} from './edit-mode'

// Re-export the patch applicator for the same reason as the route
// surface above: `@ship/sdk-client`'s SSE layer imports
// `@ship/sail-runtime/patch`, and the importmap points that subpath at
// the single runtime.js bundle — so the symbols must be reachable here.
export { applyPatch, applyPatches, tokenizePath } from './patch'
export type { Patch, PatchOp, PathToken } from './patch'

// Daemon ↔ runtime data contract. The committed `./generated.d.ts` is a
// hand-maintained mirror of the Rust wire types in this repo (no `gen-types`
// task exists; `./generated.ts` is gitignored). Keep `.d.ts` in sync when
// the Rust wire types grow a new field the runtime depends on — the
// typecheck script will catch drift. Route-system contract types only — Ship
// daemon-domain data (git/agents/account) lives in @ship/sdk-client, never
// the framework.
export type {
  ManifestRail,
  RailMeta,
  ResolveError,
  ResolvedRoute,
  RouteListEntry,
  RouteManifest,
} from './generated'
