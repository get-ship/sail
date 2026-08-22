/**
 * @ship/sail-runtime — package root (tier B author surface).
 *
 * Pedagogical / intentional author imports only. The allowlist lives in
 * `../author-surface.json` and is rendered on /sail/reference/runtime.
 * Rule (TS-1 WI2.5 / freeze): if the skill/surface does not teach it, it
 * is not public — host, emit ABI, and untaught utilities live on
 * `@ship/sail-runtime/internal` or a dedicated subpath (`./route`,
 * `./patch`, `./server`, `./framework`, …).
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

// ── Reactivity ─────────────────────────────────────────────────────────────
export { signal, derived, effect, batch, untrack, isSignal } from './signal'
export type { Signal, Derived } from './signal'

export {
  onCleanup,
  provideContext,
  injectContext,
  contextKey,
  SailMountOutsideSetupError,
} from './effect-scope'
export type { EffectScope, ContextKey } from './effect-scope'
// Ambient controller lifecycle: free name `mount` is this export, aliased at inject.
export { mount as mountAmbient } from './effect-scope'

export { debounce, interval } from './timing'
export { persisted } from './persisted'
export type { PersistedOpts } from './persisted'

// ── Controller ─────────────────────────────────────────────────────────────
export { controller } from './controller-resolver'
export type { ControllerExport } from './controller-resolver'
export type {
  ControllerDef,
  ControllerSetup,
  ControllerScope,
  ReadonlyState,
  ReadonlyView,
  MutableState,
} from './controller'

// Host capability types — generated Scope .d.ts + /sail/language/capabilities.
// SailHost is opaque (branded); unwrapSailHost is framework-only on /internal.
export type { Host } from './host'
export type { SailHost } from './sail-host'
export { scheduleFrame, asSailHost } from './sail-host'
export type { RouteSignals } from './route-signals'
// Pure-safe host / media surface growth (SPEC-L2-PACKAGE-1 P1).
export { objectUrlFromBytes, revokeObjectUrl } from './object-url'
export { utf8ToBase64 } from './base64'
export {
  defineHostProperty,
  hostGetAttr,
  hostHasAttr,
  hostSetAttr,
  hostDispatch,
  observeContentBox,
  hostIsDarkMode,
  hostCssVar,
  hostFindById,
  hostCreateElement,
  hostObserveRootAttrs,
  hostListenDocument,
  observeHostAttrs,
} from './host-property'
export type { HostPropertyDesc } from './host-property'

// ── Stores ─────────────────────────────────────────────────────────────────
export {
  defineStore,
  useStore,
  provideStore,
  peekStore,
  snapshotStore,
} from './store'
export type { StoreDef, StoreInstance } from './store'

// ── Compositions ───────────────────────────────────────────────────────────
export { form, selection, filteredList } from './compositions'
export type { FormOptions, ListSource } from './compositions'

// ── Data ───────────────────────────────────────────────────────────────────
// Values live on `@ship/sail-runtime/query` (a separate singleton).
// Types stay here so authored `import type` does not pull the cache engine.
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
  OptimisticHandle,
} from './query'

// ── Theme ──────────────────────────────────────────────────────────────────
export {
  theme,
  setThemePersist,
  hydrateTheme,
  clearThemeOverrides,
  clearThemeScope,
  lowerThemeDoc,
} from './theme'
export type {
  Theme,
  ThemePatch,
  ThemeTokenName,
  ThemeState,
  ThemePersist,
  ThemeDoc,
  ThemeApplyScopeOptions,
  ThemeScopePatch,
  LoweredThemeToken,
  LowerThemeResult,
} from './theme'

// ── Motion (scrub lane; Style motion is template/A) ────────────────────────
// Values live on `@ship/sail-runtime/motion`. Types stay on the language barrel.
export type { SailTimelineError } from './timeline'
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
export type { ClockOptions, ClockSignal, ScrollProgressOptions, SpringOptions } from './drivers'

// ── Actions ────────────────────────────────────────────────────────────────
// Value exports live on `@ship/sail-runtime/actions` (a separate singleton).
// Re-exporting them here pulled zoom-pan / virtual-row into every page.
export type { Action, ActionResult } from './action'
export type { Disposer } from './bind'
export type {
  AnchorPlacement,
  ClipboardParam,
  HotkeyEntry,
  HotkeyHandler,
  HotkeysMap,
  InViewHandler,
} from './actions'

// ── Errors ─────────────────────────────────────────────────────────────────
export {
  attachBoundary,
  detachBoundary,
  findNearestBoundary,
  runProtected,
} from './error-boundary'
export type { BoundaryHandle, BoundarySource } from './error-boundary'

// ── Components (pack authors) ──────────────────────────────────────────────
export { component, prop, defineComponent } from './component'
export type {
  PropDef,
  ComponentDef,
  ComponentSetupContext,
  ComponentFactory,
  ComponentSchema,
} from './component'

// Ambient navigate (S5.2) — host router port; inject via RUNTIME_AMBIENT.
export { navigate } from './navigate'
export type { NavigateOpts } from './navigate'

// Theme session writers on runtime ambient (SPEC-AMBIENT-SURFACE-1 §3).
// THEMES catalog + guards ride here too so stdlib controllers (theme-picker)
// read the same ambient names the inject table declares.
export {
  applyTheme,
  persistTheme,
  readStoredTheme,
  applyMode,
  bootTheme,
  persistMode,
  readStoredMode,
} from './theme-session'
export {
  THEMES,
  isValidTheme,
  themeSupportsMode,
  readModePreference,
  subscribeSystemPreference,
} from '@ship/tokens/themes'
