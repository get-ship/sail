// THE COMPILER'S ABI. Not the language.
//
// Everything here is something the EMITTER writes into generated code. No human
// types `bindFor` or `runAction`, and no agent should ever see them offered as
// the front-door API — they sit on this subpath so the root can stay a teachable
// language surface (~ten primitives + compositions + theme + public actions).
//
// Rule:
//
//     the COMPILER imports from `@ship/sail-runtime/internal`
//     HUMANS import from `@ship/sail-runtime`
//
// This file is a CURATED list, not a total re-export of the root alone. The
// ambient vocabulary (signal, controller, …) is re-exported from the root so
// the compiler's inject path (`RUNTIME_SPECIFIER`) and hand-written
// controllers resolve the same names; ABI-only symbols are named explicitly
// below so a missing export is a compile-time miss rather than a silent total
// re-export of every root symbol forever.
//
// When the emitter reaches for a new symbol: add it here AND keep it OFF the
// root. The abi_boundary tests pin both sides.

// ── Language surface the ambient inject / emit also needs ──────────────────
// Controllers and compiled modules import these through `/internal` (see
// `runtime_ambient::RUNTIME_SPECIFIER`). Re-exporting the root keeps one
// source of truth for the public names.
export * from './index'

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

// ── ABI-only: control flow ─────────────────────────────────────────────────
export { bindFor, bindForSignal } from './for'
export type { BindForOptions, RenderedItem } from './for'
export { bindIf } from './if'
export type { BindIfOptions, BranchRender } from './if'
export { bindWhileLoading, bindWhileError } from './while'
export type {
  BindWhileLoadingOptions,
  BindWhileErrorOptions,
} from './while'
export { bindSlotEmpty } from './slot-empty'

// ── ABI-only: action runner (use: / bind:this) ─────────────────────────────
export { runAction } from './action'

// ── ABI-only: template / hydrate / motion plumbing ─────────────────────────
export { __tpl } from './tpl'
export { delegateEvents } from './delegate'
export {
  markHydrateText,
  markHydrateAttr,
  markHydrateBoundary,
} from './hydrate-mark'
export { __sail_motion_attach } from './motion'
export type { MotionAttachConfig, MotionConfig } from './motion'
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
