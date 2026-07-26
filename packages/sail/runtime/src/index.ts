/**
 * @ship/sail-runtime — public LANGUAGE surface.
 *
 * This is the front door for humans and agents. The compiler's ABI
 * (`bindFor`, `runAction`, `__tpl`, `__sui_motion_attach`, …) lives on
 * `@ship/sail-runtime/internal` — if you are typing a symbol from there,
 * you are hand-writing something the emitter writes.
 *
 * Surface map (author-facing):
 *   - Reactivity:    `signal`, `derived`, `effect`, `batch`, `untrack`
 *                    (+ types `Signal`, `Derived`)
 *   - Controller:    `controller`, `resolveController`, lifecycle
 *                    (`onCleanup`, ambient `mount` → `mountAmbient`)
 *   - Data:          `query`, `mutation`, compositions (`form`, …)
 *   - Theme:         `theme`, `setThemePersist`, `hydrateTheme`
 *   - Actions:       the 12 public `use:` names (full set on `/actions`)
 *   - Motion:        `timeline`, drivers (`clock`, `scrollProgress`, …)
 *   - Host:          `mount` (route into DOM), types authors need
 *
 * ABI / DOM binders / emit plumbing: `@ship/sail-runtime/internal`.
 *
 * Remainder still on this barrel (host/advanced; not yet moved):
 *   host-api helpers, defineComponent/camelOf, slot projection, decode*,
 *   error-boundary, template-hook registry, route/patch/edit-mode re-exports
 *   (importmap singleton reachability), adoptSuiSheets.
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

// Author lifecycle. Scope plumbing the synthetic emit needs
// (`createScope`/`withScope`/`walkScope`) lives on `/internal` only —
// humans write `onCleanup` / ambient `mount(fn)`, never build scopes by hand.
export {
  onCleanup,
  provideContext,
  injectContext,
  contextKey,
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
// like RouterPort — replaces the old window.__ship* globals.
export { setHostContext, hostContext } from './host-context'
export type { HostContext } from './host-context'
export type { Host } from './host'

// The sanctioned runtime theme-write hatch — `theme.preview/commit/reset/
// current`. A plain ambient API (NOT ctx-scoped): a theme write is a GLOBAL
// `:root` write of DECLARED tokens, the `:root`-targeting sibling of the
// compiler's signal→`(--x)` reflection. `setThemePersist` is the host
// persistence seam (dev file-write / live backend); `hydrateTheme` is the
// optional live-persistence boot step. See theme.ts.
export { theme, setThemePersist, hydrateTheme, clearThemeOverrides } from './theme'
export type { Theme, ThemePatch, ThemeTokenName, ThemeState, ThemePersist } from './theme'

// Compositions — plain functions over the primitives. Part of the LANGUAGE, not a
// package you install: `form()` is no more optional than `signal()`.
export { form, selection, filteredList } from './compositions'
export type { FormOptions, ListSource } from './compositions'

// `Action` is the author type for custom `use:` functions. The runner
// (`runAction`) is ABI — on `/internal` only.
export type { Action, ActionResult } from './action'
// Disposer is the cleanup shape actions / effects return — author-facing type.
export type { Disposer } from './bind'

// Public `use:` surface — the 12 app-author actions taught in the skill/docs.
// The full set (stdlib-internal stamps included) lives on
// `@ship/sail-runtime/actions`; the compiler auto-imports *all* actions from
// that subpath so stdlib stamps resolve without polluting this front door.
// Drift is guarded by `public_action_surface_is_curated` in abi_boundary.rs.
export {
  anchor,
  autosize,
  clickOutside,
  contextMenu,
  dropzone,
  icon,
  primeIconCache,
  menu,
  rect,
  tabs,
  tipDescribe,
  virtualRow,
  zoomPan,
} from './actions'
export type { AnchorPlacement } from './actions'


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

export type { ComponentExample } from './component-example'

// Host-element helpers for hand-written custom elements (rare; most
// authors use `component()`). ABI DOM binders stay on `/internal`.
export {
  emit,
  expose,
  attachHostApi,
  bindNativeEvent,
  slotted,
  bindHostAttr,
} from './host-api'

// Backbone #2 — virtualization — lives on the `@ship/sail-runtime/virtual`
// subpath, NOT this barrel (#297): re-exporting it here dragged
// `@tanstack/virtual-core` (~21KB minified, a third of the bundle) into the
// always-loaded runtime singleton. Windowing consumers import
// `createVirtualizer` / `Virtual` / `VirtualItem` from the subpath; their
// bundles inline the adapter + engine while the reactive core stays the
// external singleton (see src/virtual.ts).

// Slot helpers for advanced component authors (projection). List/control-
// flow binders (`bindFor`/`bindIf`/…) are ABI on `/internal`.
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

export { debounce } from './signal'

// query()/mutation() — the read/write primitives. `live()` is DELETED
// (data-arch-seed.md, "Owner RULINGS 2026-07-18"): its PULL half is
// query() (Promise fetcher) and its PUSH half folded into query() by
// type-directed routing at the runtime seam (LiveSource fetcher). The
// loading/ok/error + push-source types, and `unwrap`/`unwrapOr`, are
// owned by query.ts now.
// `__invalidate` is compiler-only ambient (rewrite of `invalidate(ref)`) —
// kept here so `/internal`'s re-export of the root still carries it for
// ambient inject; authors never type it.
export { query, queryKeyed, mutation, __invalidate, unwrap, unwrapOr } from './query'

export { wireFormMutation } from './mutation'
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

// Flat host-inject port — empty by default; apps augment named providers.

export { component, prop } from './component'
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
// context/setup/module types. `component` itself is exported above;
// these are the types the mount-form's setup + default export carry.
export type { MountContext, MountSetup, MountModule } from './component-mount'

// Template-hook registry — author extension point for `<use-*>`.
// Lookup used by the emitter (`getTemplateHook`) is also on `/internal`.
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

// The choreography lane. Ambient `fx:` attaches via `__sui_motion_attach`
// (ABI on `/internal`); `timeline()` is the author-facing scrub/sequence API.
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
// Residual shared-sheet adoption for opt-in shadow containment wrappers
// (foreign content — not L1). No-ops without a ShadowRoot. Kept on the client
// entry so legacy/compiled call sites that still import it resolve.
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
