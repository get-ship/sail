// controller.ts — typed contract for the five-section controller shape.
//
// Per the "state is declarative-only" ADR, controllers no longer have
// a `state:` section — state is declared exclusively in the template
// via `<state name="X" value="..." />`. A controller has five
// sections: `derived`, `on`, `use`, `mount`, `unmount`. The state-
// shape generic is gone; mutation/read views are derived from the
// `Scope` generic, which is what the template declares.
//
// Hooks (`ControllerHook`) keep an internal state-bearing shape — they
// declare their own writable signals as part of their composition
// surface. Hook state is namespaced under the `use:` key and never
// flat-merges into the host scope.

import type { Signal } from './signal'
import type { ShipHost, Clients } from './ship'
// Slice 2 — SUI-local `RouteSignals` (see ./route-signals for the
// decoupling rationale). Replaces the `@ship/deck-router` import.
import type { RouteSignals } from './route-signals'

// ---------------------------------------------------------------------------
// State-shape extraction over Scope
// ---------------------------------------------------------------------------

/**
 * Extract the state-key subset of `Scope`. A state key is a scope
 * identifier whose value is a writable `Signal<T>` — that's how the
 * template's `<state>` directive contributes to scope. Read-only
 * derived/data identifiers and non-signal values fall through.
 */
// A signal of any value type. `Signal<unknown>` does NOT match a concrete
// signal: `Signal` is invariant in `T` (it both reads `(): T` and writes
// `set(v: T)`), so `Signal<number> extends Signal<unknown>` is *false*.
// Using it as the filter constraint silently collapsed `StateKeys` to
// `never` — `MutableState` had no keys, and every handler cast `s as any`
// to touch its own state. `Signal<any>` matches a signal of any value type.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnySignal = Signal<any>

export type StateKeys<Scope> = {
  [K in keyof Scope]: Scope[K] extends AnySignal ? K : never
}[keyof Scope]

/** The signal-typed value at scope key K, or never. */
export type SignalValueAt<Scope, K extends keyof Scope> = Scope[K] extends Signal<infer V>
  ? V
  : never

// ---------------------------------------------------------------------------
// Scope shapes
// ---------------------------------------------------------------------------

/**
 * Read-only access to a state key from declarative contexts (templates,
 * `derived` sections, controller hooks). Callable to read; no `.set`.
 */
export interface ReadonlyView<T> {
  /** Read the current value (and track in any active reactive scope). */
  (): T
}

/**
 * The "this view's state, as seen from declarative contexts."
 *
 * Each state key (i.e. each scope key whose value is a `Signal`) is
 * exposed as a callable getter, never as a writable proxy. Used in
 * `derived` section signatures and as a constraint on what templates
 * can read.
 */
export type ReadonlyState<Scope> = {
  readonly [K in StateKeys<Scope>]: ReadonlyView<SignalValueAt<Scope, K>>
}

/**
 * The "this view's state, as seen from imperative contexts" (handlers,
 * `mount`, `unmount`).
 *
 * Each state key is a plain assignable property: read returns the
 * current value, write replaces it. The runtime translates the
 * assignment into a `signal.set()` call.
 */
export type MutableState<Scope> =
  // Writable state keys: read returns the unwrapped value, write forwards to
  // `signal.set()`. (`state.count = state.count + 1` works.)
  {
    [K in StateKeys<Scope>]: SignalValueAt<Scope, K>
  } & {
    // Every non-state scope key is readable as-is — a `derived` stays a
    // callable `ReadonlyView<T>` (read it by calling, `s.label()`), a
    // `<data>` live-signal stays its live shape. The runtime proxy falls
    // through to the base scope for these. Readonly: handlers mutate only
    // state keys.
    readonly [K in Exclude<keyof Scope, StateKeys<Scope>>]: Scope[K]
  } & {
    // The ambient view context the proxy also exposes, matching the runtime
    // (`baseScope` carries `ship` / `route` / `navigate`).
    readonly ship: ShipHost
    readonly clients: Clients
    readonly route: RouteSignals
    readonly navigate: (path: string) => void
    // Explicit signal access for the escape-hatch cases the assignment style
    // can't express (handing a `Signal<T>` to a callback, `.set(updater)`).
    readonly $signals: { readonly [K in StateKeys<Scope>]: Scope[K] }
  }

/**
 * The shared view scope, as destructured by each controller section.
 *
 * Includes `ship`, the reactive `route` proxy, the `<data>`-declared
 * signals from the template, the `<state>`-declared writable signals
 * (read-only here), and the controller-hook contributions.
 *
 * `route` is a `RouteSignals` whose `params.<name>` and
 * `query.<name>` are themselves signals — reading them inside a
 * `derived` body or a `<data live>` source thunk subscribes the
 * surrounding scope, so URL changes re-fire the dependent
 * computation.
 */
export type ViewScope<Scope = object> = Scope & {
  ship: ShipHost
  clients: Clients
  route: RouteSignals
}

// ---------------------------------------------------------------------------
// Section types
// ---------------------------------------------------------------------------

/** A section-init can be a static value or a factory that receives scope. */
export type SectionInit<V, Scope> = V | ((scope: Scope) => V)

/**
 * `on` section signature. Each handler receives a writable state
 * mutator as its first argument and any handler-specific args after.
 *
 * Handlers may also accept the broader scope (with state attached) as
 * their first argument when they need to read other signals or call
 * `ship.*` directly. The runtime resolver discriminates by the first
 * parameter's shape.
 */
export type OnSection<Scope> = Record<
  string,
  // Single form — `state` is `MutableState<Scope>`, which now exposes
  // writable state keys, readonly derived/data/scope keys, and ambient
  // `ship`/`route`/`navigate`. A previous two-member union (state-only OR
  // full-scope) defeated contextual inference: `(s) =>` fell back to
  // implicit `any`, so authors annotated `type S = MutableState<Scope>` and
  // then cast. One signature infers cleanly.
  //
  // `any[]` (not `unknown[]`) for the trailing args is deliberate: handlers
  // declare typed args, and under contravariant param checks those aren't
  // assignable to a `(...args: unknown[])` constraint, so we widen to `any`
  // so authors can name argument types without an intermediate cast.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (state: MutableState<Scope>, ...args: any[]) => void | Promise<void>
>

/**
 * `use` section signature. Each entry is a controller hook (see
 * `ControllerHook`). Results are namespaced by key in the resolved
 * view scope.
 */
export type UseSection = Record<string, ControllerHook<any, any>>

/**
 * `mount` lifecycle. Runs once after the rendered DOM is attached and
 * all signals/handlers/hooks are wired. May return a cleanup function
 * that is called on disconnect (equivalent to pushing to cleanups).
 *
 * `state` is the writable proxy — the same shape `on:` handlers see.
 * Two access patterns:
 *
 *   1. **Assignment style** (ergonomic for primitives):
 *      `state.viewportH = el.clientHeight`
 *      `state.count++`
 *      The proxy intercepts the write and forwards to `signal.set()`.
 *      Reads return the unwrapped value (`state.count` is the number,
 *      not the signal), which is what makes `state.count++` work.
 *
 *   2. **Explicit signal access** via `state.$signals`:
 *      `state.$signals.viewportH.set(el.clientHeight)`
 *      `state.$signals.viewportH(prev => prev + 1)`
 *      Use this when a primitive's `.set` would otherwise be tempting
 *      (`state.viewportH.set(...)` throws because the value-side read
 *      returned a number) or when you want to hand a `Signal<T>` to a
 *      callback that closes over it.
 *
 * Non-signal entries from the controller's `scope: () => ({...})`
 * factory (helper functions, plain values) are directly accessible
 * via the proxy fallthrough — `state.reset()` resolves to the
 * factory-returned arrow. The compiler emits both groups into the
 * per-file `Scope` interface in `<file>.sui.d.ts`.
 */
export type MountSection<Scope> = (
  state: MutableState<Scope>,
  el: HTMLElement,
) => (() => void) | void | Promise<void>

/**
 * `unmount` lifecycle. Runs once on disconnect; tear down whatever
 * `mount` set up. Receives the same `Scope`-typed state view as `mount`.
 */
export type UnmountSection<Scope> = (state: MutableState<Scope>) => void

// ---------------------------------------------------------------------------
// The controller surface
// ---------------------------------------------------------------------------

/**
 * The full shape of a controller: five optional sections, each with a
 * static or factory form where applicable.
 *
 * State is **not** a controller section — declare it in the template
 * via `<state name="X" value="..." />`. Computed-initial cases move
 * to `mount` (`state.X = crypto.randomUUID()`).
 *
 * @typeParam Scope Template-declared scope (`<data>`, `<state>`,
 *   `<use-* as=>`) plus `use:` hook contributions.
 */
export interface ControllerShape<Scope = object> {
  /**
   * Factory that produces typed scope contributions: signals,
   * derived, hooks, shared-harness/consumer wrappers. The returned
   * object is merged into the template's scope. Runs before any
   * other section so derived/on/use can reference its entries.
   *
   * Replaces the historical `state:` + `derived:` + `use:` split for
   * tier-2+ controllers. Author writes idiomatic TypeScript inside
   * the factory; types flow through inference.
   */
  scope?: (init: ViewScope<Scope>, host?: Element) => Record<string, unknown>

  /**
   * Event handlers. The **only** section in a controller that can
   * mutate state.
   */
  on?: OnSection<Scope>

  /**
   * Composition of controller hooks. Static or factory form; factory
   * form receives the merged scope.
   */
  use?: SectionInit<UseSection, ViewScope<Scope>>

  /** Runs once after mount; receives writable state. */
  mount?: MountSection<Scope>

  /** Runs once on disconnect; receives writable state. */
  unmount?: UnmountSection<Scope>
}

/**
 * The canonical controller shape — single call, precise, cast-free.
 *
 * Both type parameters are INFERRED from the `def` object, so authors
 * write `controller({ scope: (ctx: ViewScope<Scope>) => ({…}) })` with no
 * explicit type argument and no currying:
 *   - `Ctx` is inferred from the `scope` factory's PARAMETER annotation
 *     (the compiler-generated `ViewScope<Scope>` the author imports). This
 *     sidesteps TypeScript's all-or-nothing type-argument rule — nothing is
 *     supplied explicitly, so nothing forces a second call.
 *   - `SC` is inferred from the `scope` factory's RETURN — the controller's
 *     own TS is the source of truth for `signal`/`derived`/`ref`/`live`
 *     types, so handler state over `Ctx & SC` is precise.
 *
 * `scope` is required (it's where `Ctx` is declared); a pure-lifecycle
 * controller returns `({})`.
 */
export interface ScopedControllerDef<Ctx, SC extends Record<string, unknown>> {
  scope: (init: Ctx, host?: Element) => SC
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  on?: Record<string, (state: MutableState<Ctx & SC>, ...args: any[]) => void | Promise<void>>
  use?: SectionInit<UseSection, Ctx & SC>
  mount?: (state: MutableState<Ctx & SC>, el: HTMLElement) => (() => void) | void | Promise<void>
  unmount?: (state: MutableState<Ctx & SC>) => void
}

/**
 * The two valid forms of a controller export — an object (tier 2) or
 * a function (tier 3). The function form's argument is `{ ship }`
 * only — template scope is not available at controller-instantiation
 * time, only inside section factories.
 */
export type ControllerDef<Scope = object> =
  | ControllerShape<Scope>
  | ((ctx: { ship: ShipHost; clients: Clients }) => ControllerShape<Scope>)

// `controller(def)` runtime helper lives in `controller-resolver.ts`.

// ---------------------------------------------------------------------------
// Controller hooks
// ---------------------------------------------------------------------------

/**
 * Authoring contract for a controller hook.
 *
 * Hooks are composable bundles; unlike controllers, they **do** declare
 * their own `state:` section because hook-local state is what makes
 * them coherent slices (`useFilteredList` owns its `query`,
 * `useSelection` owns its `selectedId`). Hook state is namespaced
 * under the `use:` key in the consuming controller — there is no
 * automatic flat-merge — so two hooks both declaring `state.query`
 * coexist as `<keyA>.query` and `<keyB>.query`.
 *
 * @typeParam S Hook-internal state shape.
 * @typeParam Scope Extra scope (usually `object` for hooks).
 */
export interface ControllerHook<
  S extends object = object,
  Scope = object,
> {
  /**
   * Hook-internal state. Static initial values; the runtime constructs
   * a `Signal<T>` per key.
   */
  state?: S | ((scope: { ship: ShipHost; clients: Clients } & Scope) => S)
  /** Pure memos over the merged hook scope (state + outer scope). */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  derived?: Record<string, (scope: any) => unknown>
  /** Event handlers; only section that mutates hook state. */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  on?: Record<string, (state: any, ...args: any[]) => void | Promise<void>>
  /** Hook composition (a hook may itself compose other hooks). */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  use?: Record<string, ControllerHook> | ((scope: any) => Record<string, ControllerHook>)
  /** Runs once after mount; receives writable hook state. */
  mount?: (state: S, el: HTMLElement) => void
  /** Runs once on disconnect; receives writable hook state. */
  unmount?: (state: S) => void
}

// Template-hook contracts (`TemplateHook`, `TemplateHookContext`,
// `TemplateHookAttrKind`) live in `./template-hook.ts`.
export type {
  TemplateHook,
  TemplateHookAttrKind,
  TemplateHookContext,
} from './template-hook'
