// THE COMPILER'S ABI + host embedder surface. Not the language.
//
// Everything here is something the EMITTER writes into generated code, or a
// host/framework symbol that must not appear on the author front door. No
// human types `bindFor` or `runAction`, and no agent should ever see them
// offered as the front-door API — they sit on this subpath so the root can
// stay a teachable language surface (author-surface.json / TS-1 WI2.5).
//
// Rule:
//
//     the COMPILER imports from `@ship/sail-runtime/internal`
//     HUMANS import from `@ship/sail-runtime`
//     HOSTS / pack engines may use `/internal` or dedicated subpaths
//
// This file is a CURATED list, not a total re-export of the root alone. The
// ambient vocabulary (signal, controller, …) is re-exported from the root so
// the compiler's inject path (`RUNTIME_SPECIFIER`) and hand-written
// controllers resolve the same names; ABI-only and host-only symbols are
// named explicitly below so a missing export is a compile-time miss rather
// than a silent total re-export of every root symbol forever.
//
// When the emitter reaches for a new symbol: add it here AND keep it OFF the
// root. The abi_boundary tests pin both sides.

// ── Language surface the ambient inject / emit also needs ──────────────────
// Controllers and compiled modules import these through `/internal` (see
// `runtime_ambient::RUNTIME_SPECIFIER`). Re-exporting the root keeps one
// source of truth for the public names.
export * from './index'

// Virtualizer, motion attach, and query live on their own singletons
// (`/virtual`, `/motion`, `/query`). Hello-world must not download them.

// ── ABI-only: DOM binders ──────────────────────────────────────────────────
export {
  bindText,
  bindShow,
  bindAttr,
  bindProp,
  bindDynamic,
  bindValue,
  bindChecked,
  bindHtml,
} from './bind'

// ── ABI-only: events ───────────────────────────────────────────────────────
export { on, matchesKey } from './on'
export type { OnOptions, Modifier } from './on'
export { invokeCallback } from './callback'

// ── ABI-only: control flow ─────────────────────────────────────────────────
export { bindFor, bindForSignal } from './for'
export type { BindForOptions, RenderedItem } from './for'
export { bindIf } from './if'
export type { BindIfOptions, BranchRender } from './if'
export { bindWhen, acquireMediaQuery } from './when'
export type { BindWhenOptions } from './when'
export { bindSlotEmpty } from './slot-empty'

// ── ABI-only: action runner (use:) ─────────────────────────────────────────
export { runAction } from './action'

// ── ABI-only: template / hydrate / motion plumbing ─────────────────────────
export { __tpl } from './tpl'
export { delegateEvents } from './delegate'
export {
  markHydrateText,
  markHydrateAttr,
  markHydrateBoundary,
} from './hydrate-mark'
export { runEnter, runLeave, cancelTransitions, setTransitionRunner } from './transition-runner'

// ── ABI-only: form / dialog / popover wire ─────────────────────────────────
export { wireFormMutation } from './mutation'
export { bindOpen } from './enhance-dialog'
export type { DialogOptions } from './enhance-dialog'
export { bindPopover } from './enhance-popover'
export type { PopoverOptions } from './enhance-popover'
export { bindRawHtml } from './raw-html'
export { morph } from './morph'
export type { MorphOptions } from './morph'

// ── ABI-only: mount / scope plumbing the synthetic + CE paths use ──────────
export { mountComponent } from './compose'
export { createMountState, componentMountState, pickSignals, SIGNALS_KEY } from './state-proxy'
export {
  createScope,
  createChildScope,
  withScope,
  currentScope,
  walkScope,
} from './effect-scope'

// ── Untaught reactivity utilities (not tier B; pack engines may import) ───
export { selector, withHost } from './signal'

// ── Host: route mount into the DOM (deck-router / framework) ───────────────
export { mount } from './mount'
export type { RoutePayload, MountOptions, MountHandle, SailLoader } from './mount'

// ── Host: context + edit/annotation mode ───────────────────────────────────
export { setHostContext, hostContext } from './host-context'
export type { HostContext } from './host-context'
export { setHostRoute, hostRoute } from './host-route'
export type { HostRoute } from './host-route'
export type { Host } from './host'
// Framework-only SailHost unwrap (not tier B author root; DOM lib types).
export { unwrapSailHost } from './sail-host-dom'
export {
  editModeForRoute,
  toggleEditMode,
  setEditMode,
  annotationModeForRoute,
  toggleAnnotationMode,
  setAnnotationMode,
} from './edit-mode'

// ── Host: patch protocol (also `@ship/sail-runtime/patch`) ─────────────────
export { applyPatch, applyPatches, tokenizePath } from './patch'
export type { Patch, PatchOp, PathToken } from './patch'

// ── Host / pack: CE host API + component plumbing ──────────────────────────
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
export { camelOf } from './component'
export { clientRootScope, elementScopeOf } from './element-scope'
export { resolveController } from './controller-resolver'
export type { ResolverContext, ResolvedBindings } from './controller-resolver'
export type {
  TemplateHook,
  TemplateHookAttrKind,
  TemplateHookContext,
} from './controller'
export type { MountContext, MountSetup, MountModule } from './component-mount'
export type { ComponentExample } from './component-example'
export {
  decodeString,
  decodeNumber,
  decodeBoolean,
  decodeJson,
  decodeEnum,
} from './decode'
export type { Decoder } from './decode'

// ── Host: template-hook registry (also `@ship/sail-runtime/template-hooks`) ─
export {
  registerTemplateHook,
  getTemplateHook,
  clearRegistry,
  listRegisteredHooks,
} from './template-hooks/registry'
export type { KbdShortcut } from './template-hooks/kbd'

// ── Host: scroll-lock helpers used by overlay actions ──────────────────────
export {
  focusableWithin,
  FocusReturn,
  acquireScrollLock,
  releaseScrollLock,
} from './scroll-lock'

// ── Host: SSR store seed / snapshot (server entry also exports these) ──────
export {
  setStoreSSRPayload,
  clearStoreSSRPayload,
  readStoreSSRSeed,
  hydrateStoreSSRPayload,
  snapshotScopeStores,
  SSR_PAYLOAD_GLOBAL,
} from './store'

// ── Host: route signals factory (also `@ship/sail-runtime/route`) ──────────
export { route, routeWithUpdate } from './route'
export type { RouteHandle } from './route'
export type { RouteSignals } from './route-signals'

// ── Host: generated route/manifest types ───────────────────────────────────
export type {
  ManifestRail,
  RailMeta,
  ResolveError,
  ResolvedRoute,
  RouteListEntry,
  RouteManifest,
} from './generated'

