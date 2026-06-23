// controller-hook-resolver.ts — runtime backing for `ControllerHook`.
//
// Hooks keep their own state-bearing shape (unlike outer controllers,
// which use the template's `<state>` directives). This module owns the
// hook-state construction + recursive hook composition; the outer
// controller resolver delegates here from its `use:` handling.

import { signal, derived as memo } from './signal'
import type { Signal } from './signal'
import type { ControllerHook, ReadonlyView } from './controller'
import type { ResolverContext, ResolvedBindings } from './controller-resolver-types'

/**
 * Resolve a `ControllerHook` into its flat bindings record. Hooks
 * differ from controllers in one place: they own a `state:` literal
 * that the resolver turns into writable signals at hook-mount time.
 * Everything else (derived / on / use / lifecycle) walks identically
 * to the outer controller resolver.
 *
 * The returned bindings include the hook's state signals (so the
 * template can reach `<key>.<stateName>` for two-way binding), its
 * derived memos, its handler closures, and any nested hook namespaces.
 */
export function resolveHook(
  hook: ControllerHook,
  ctx: ResolverContext,
  outerScope: Record<string, unknown>,
): ResolvedBindings {
  const hookState = resolveHookState(hook.state, { ship: ctx.ship, clients: ctx.clients ?? {}, ...(ctx.clients ?? {}), ...outerScope })
  const stateSignals: Record<string, Signal<unknown>> = {}
  for (const [k, v] of Object.entries(hookState)) {
    stateSignals[k] = signal(v)
  }
  const baseScope: Record<string, unknown> = {
    ship: ctx.ship,
    clients: ctx.clients ?? {},
    ...(ctx.clients ?? {}),
    route: ctx.route ?? {},
    navigate: ctx.navigate ?? ((_path: string) => {}),
    ...outerScope,
    ...stateSignals,
  }
  const writableProxy = buildHookWritableProxy(stateSignals, baseScope)
  const declScope: Record<string, unknown> = { ...baseScope }

  const deriveds: Record<string, ReadonlyView<unknown>> = {}
  for (const [k, fn] of Object.entries(hook.derived ?? {})) {
    deriveds[k] = memo(() => (fn as (s: unknown) => unknown)(declScope))
  }
  Object.assign(declScope, deriveds)

  const onHandlers: Record<string, (...args: unknown[]) => unknown> = {}
  for (const [k, fn] of Object.entries(hook.on ?? {})) {
    onHandlers[k] = (...args: unknown[]) =>
      (fn as (state: unknown, ...rest: unknown[]) => unknown)(writableProxy, ...args)
  }

  const useResults: Record<string, ResolvedBindings> = {}
  const useSection =
    typeof hook.use === 'function'
      ? (hook.use as (s: unknown) => Record<string, ControllerHook>)(declScope)
      : (hook.use as Record<string, ControllerHook> | undefined) ?? {}
  for (const [k, sub] of Object.entries(useSection)) {
    useResults[k] = resolveHook(sub, ctx, declScope)
  }

  return {
    ...stateSignals,
    ...deriveds,
    ...onHandlers,
    ...useResults,
  }
}

function resolveHookState(
  section: ControllerHook['state'],
  scope: Record<string, unknown>,
): Record<string, unknown> {
  if (section === undefined) return {}
  if (typeof section === 'function') {
    return (section as (s: unknown) => Record<string, unknown>)(scope)
  }
  return section as Record<string, unknown>
}

function buildHookWritableProxy(
  stateSignals: Record<string, Signal<unknown>>,
  baseScope: Record<string, unknown>,
): Record<string, unknown> {
  return new Proxy({} as Record<string, unknown>, {
    get(_target, key: string) {
      if (key in stateSignals) return stateSignals[key]!()
      return baseScope[key]
    },
    set(_target, key: string, value: unknown) {
      if (key in stateSignals) {
        stateSignals[key]!.set(value)
        return true
      }
      console.warn(
        `[ship-set] hook handler tried to write s.${key} = … but ${key} is not a hook state key. The assignment was ignored.`,
      )
      return true
    },
    has(_target, key: string) {
      return key in stateSignals || key in baseScope
    },
    ownKeys() {
      return Object.keys(stateSignals)
    },
    getOwnPropertyDescriptor(_target, key: string) {
      if (key in stateSignals) {
        return { configurable: true, enumerable: true, writable: true, value: stateSignals[key]!() }
      }
      return undefined
    },
  })
}
