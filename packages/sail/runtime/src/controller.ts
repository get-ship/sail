// controller.ts — typed contract for the controller(fn) shape.
//
// One idiom: `controller((ctx) => { ...; return { ...surface } })`. The
// setup function IS the controller — no sections. State is a plain
// `signal()` closed over by the function; the ambient `mount(fn)` and
// `onCleanup(fn)` (see `./effect-scope`) register lifecycle from
// anywhere in the setup call graph (including a composition called
// from setup); whatever the function returns becomes the merged
// template scope, handlers included, verbatim.
//
// This retired the object form (`controller({ setup, derived, on,
// mount, unmount })`) and the `use:` hook mini-DSL before it. Both
// made a framework feature out of what is now just calling a plain
// TypeScript function and returning an object — the same move,
// applied twice. Compositions (`@ship/sail-std`) are still plain
// functions: call one in the controller and the signals it returns
// are ordinary scope members. The only language type a composition
// touches is `Signal<T>`.
//
// `use:` now means one thing: the template directive that attaches
// an Action to a DOM node.

import type { Signal } from './signal'
// Slice 2 — SAIL-local `RouteSignals` (see ./route-signals for the
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
 * The "this route's state, as seen from declarative contexts."
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
 * The "this route's state, as seen from imperative contexts" (handlers,
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
    // live envelope signal stays its live shape. The runtime proxy falls
    // through to the base scope for these. Readonly: handlers mutate only
    // state keys.
    readonly [K in Exclude<keyof Scope, StateKeys<Scope>>]: Scope[K]
  } & {
    // Framework injects the proxy also exposes, matching the runtime
    // `baseScope` (`route` / `navigate`). Named providers are flat Scope
    // members (Host augmentations), not a nested bag.
    readonly route: RouteSignals
    readonly navigate: (path: string) => void
    // Explicit signal access for the escape-hatch cases the assignment style
    // can't express (handing a `Signal<T>` to a callback, `.set(updater)`).
    readonly $signals: { readonly [K in StateKeys<Scope>]: Scope[K] }
  }

/**
 * The controller's scope — now IDENTICAL to the generated `Scope`.
 *
 * @deprecated Write `Scope` directly. This is an identity alias kept so the
 * existing corpus (`controller((ctx: ControllerScope<Scope>) => …)`) keeps
 * compiling verbatim while call sites are swept; it will be deleted.
 *
 * The compiler emits the resolved capability list into `Scope` itself
 * (`route`, `navigate`, and one member per declared provider — see
 * `emit::dts::emit_dts`), so the wrapper has nothing left to add. A name a route
 * never declared is a type error, which is the point.
 */
export type ControllerScope<Scope = object> = Scope

// ---------------------------------------------------------------------------
// The controller surface — one function, one idiom
// ---------------------------------------------------------------------------

/**
 * The controller def: ONE setup function. It receives the route scope
 * (`clients`, `route`, `navigate`, plus template-declared
 * live envelope signals) and, optionally, the host element; it
 * returns the surface the template merges into scope — signals,
 * `derived()` values, plain helpers, and event handlers alike, all as
 * ordinary object properties.
 *
 * There is no separate mutation-only section: state is a `signal()`
 * closed over by this same function, and handlers returned from it
 * mutate that signal directly (`items.set([...items(), x])`) — the
 * single idiom `controller_state_set_call` (sail-lint) now enforces
 * everywhere, replacing the old proxy-vs-raw-signal split between
 * `setup`/`derived` and `on`/`mount`/`unmount`.
 *
 * Lifecycle is ambient, not a section: call `mount(fn)` (fires once
 * after this instance's DOM connects, `fn` gets the host element and
 * may return a cleanup) and `onCleanup(fn)` (runs on dispose) from
 * anywhere in the setup call graph — including a composition function
 * called from setup. See `./effect-scope`.
 *
 * Both type parameters are INFERRED, so authors write
 * `controller((ctx: ControllerScope<Scope>) => ({…}))` with no explicit type
 * argument and no currying:
 *   - `Ctx` is inferred from `setup`'s PARAMETER annotation (the
 *     compiler-generated `ControllerScope<Scope>` the author imports). This
 *     sidesteps TypeScript's all-or-nothing type-argument rule —
 *     nothing is supplied explicitly, so nothing forces a second call.
 *   - `SC` is inferred from `setup`'s RETURN — the controller's own TS
 *     is the source of truth for `signal`/`derived`/`ref`/`live`
 *     types, so the merged scope type is precise.
 */
export type ControllerSetup<Ctx, SC extends Record<string, unknown>> = (
  init: Ctx,
  host?: Element,
) => SC

/** A controller def as the resolver sees it: the setup function, untyped
 *  (the precise `Ctx`/`SC` inference lives on the `controller()` overload
 *  in `controller-resolver.ts`, not here). */
export type ControllerDef<Scope = object> = (
  init: ControllerScope<Scope>,
  host?: Element,
) => Record<string, unknown>

// `controller(def)` runtime helper lives in `controller-resolver.ts`.

// ---------------------------------------------------------------------------
// Controller hooks
// ---------------------------------------------------------------------------


// Template-hook contracts (`TemplateHook`, `TemplateHookContext`,
// `TemplateHookAttrKind`) live in `./template-hook.ts`.
export type {
  TemplateHook,
  TemplateHookAttrKind,
  TemplateHookContext,
} from './template-hook'
