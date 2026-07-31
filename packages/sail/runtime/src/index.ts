/**
 * @ship/sail-runtime — package root.
 *
 * Pedagogical / intentional author imports (tier B) are listed in
 * `../author-surface.json` and rendered on /sail/reference/runtime.
 * Emit-only ABI is `@ship/sail-runtime/internal` (tier C). Host embedders
 * also reach subpaths (`./server`, `./framework`, `./route`, …) that are
 * not the language surface.
 *
 * @packageDocumentation
 */

// Module-level state (signals/effects/registries) must be a single instance.
// Warn on duplicate loads; do not throw (dev servers may load source + bundle).
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

export {
  onCleanup,
  provideContext,
  injectContext,
  contextKey,
  SailMountOutsideSetupError,
} from './effect-scope'
export type { EffectScope, ContextKey } from './effect-scope'
// Aliased: bare `mount` below is the host route mount entry point.
export { mount as mountAmbient } from './effect-scope'

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

export { setHostContext, hostContext } from './host-context'
export type { HostContext } from './host-context'
export type { Host } from './host'

export { theme, setThemePersist, hydrateTheme, clearThemeOverrides } from './theme'
export type { Theme, ThemePatch, ThemeTokenName, ThemeState, ThemePersist } from './theme'

export { form, selection, filteredList } from './compositions'
export type { FormOptions, ListSource } from './compositions'

export type { Action, ActionResult } from './action'
export type { Disposer } from './bind'

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

export {
  focusableWithin,
  FocusReturn,
  acquireScrollLock,
  releaseScrollLock,
} from './scroll-lock'

export type { ComponentExample } from './component-example'

export {
  emit,
  expose,
  attachHostApi,
  bindNativeEvent,
  slotted,
  bindHostAttr,
} from './host-api'

export { resolveSlots, snapshotChildren, slotFilled } from './slot'
export type { SlotDecl } from './slot'

export { defineComponent, camelOf } from './component'
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

// `__invalidate` is compiler ambient only (authors never type it).
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

export type { MountContext, MountSetup, MountModule } from './component-mount'

export {
  registerTemplateHook,
  getTemplateHook,
  clearRegistry,
  listRegisteredHooks,
} from './template-hooks/registry'
export type { KbdShortcut } from './template-hooks/kbd'
import './boundary-elements'

export {
  attachBoundary,
  detachBoundary,
  findNearestBoundary,
  runProtected,
} from './error-boundary'
export type { BoundaryHandle, BoundarySource } from './error-boundary'

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

export { clock, scrollProgress, springValue } from './drivers'
export type { ClockOptions, ClockSignal, ScrollProgressOptions, SpringOptions } from './drivers'
export { mount } from './mount'
export type { RoutePayload, MountOptions, MountHandle, SailLoader } from './mount'

export type { RouteSignals } from './route-signals'

export { route, routeWithUpdate } from './route'
export type { RouteHandle } from './route'

export {
  editModeForRoute, toggleEditMode, setEditMode,
  annotationModeForRoute, toggleAnnotationMode, setAnnotationMode,
} from './edit-mode'

export { applyPatch, applyPatches, tokenizePath } from './patch'
export type { Patch, PatchOp, PathToken } from './patch'

export type {
  ManifestRail,
  RailMeta,
  ResolveError,
  ResolvedRoute,
  RouteListEntry,
  RouteManifest,
} from './generated'
