import { describe, expectTypeOf, it } from 'vitest'
import { signal, derived } from '../src/signal'
import { controller } from '../src/index'
import type { MutableState, StateKeys, ReadonlyView, ViewScope } from '../src/controller'

// Type-level spec for the handler-state arg (#114). The bug: `StateKeys`
// filtered on `Scope[K] extends Signal<unknown>`, which is FALSE for every
// concrete `Signal<T>` (Signal is invariant in T), so `MutableState<Scope>`
// resolved to `{}` — and every `on:`/`mount` handler had to cast `s as any`
// to touch its own state. These tests fail until the constraint matches any
// signal and the handler arg also exposes derived/data keys (readonly).

interface Scope {
  count: ReturnType<typeof signal<number>>
  flag: ReturnType<typeof signal<boolean>>
  label: ReadonlyView<string>
}

describe('MutableState<Scope> (handler-state arg)', () => {
  it('includes writable signal keys with their unwrapped value type', () => {
    expectTypeOf<StateKeys<Scope>>().toEqualTypeOf<'count' | 'flag'>()
    expectTypeOf<MutableState<Scope>['count']>().toEqualTypeOf<number>()
    expectTypeOf<MutableState<Scope>['flag']>().toEqualTypeOf<boolean>()
  })

  it('exposes non-state scope keys (deriveds) as readonly, still callable', () => {
    // A derived is read by calling it — the handler arg must keep it callable,
    // not pretend it is the unwrapped value (the latent bug `ds.name as string`
    // masked: `name` is a function, not a string).
    expectTypeOf<MutableState<Scope>['label']>().toEqualTypeOf<ReadonlyView<string>>()
  })

  it('canonical form: handlers get PRECISE, cast-free types inferred from scope', () => {
    // `controller({ scope: (ctx: ViewScope<Tpl>) => ({…}) })` infers both the
    // template `Ctx` (from the scope param annotation) and `SC` (from the
    // return) — the controller's own TS is the source of truth (no
    // `Signal<unknown>` from the compiler). No `<Scope>` type arg, no currying,
    // no `type S = MutableState<Scope>`, no `s as any`; values are precise.
    //
    // `derived()` returns `Derived<T> extends Signal<T>`, so it reads as its
    // unwrapped VALUE (the proxy unwraps state signals) — `s.label`, not
    // `s.label()`.
    interface Tpl { /* template-declared <data>/<state> only */ }
    controller({
      scope: (_ctx: ViewScope<Tpl>) => ({
        count: signal(0),
        flag: signal(false),
        label: derived(() => 'hello'),
        ws: derived(() => 'a' as string | null),
      }),
      on: {
        bump: (s) => {
          s.count = s.count + 1                // writable signal key -> number
          s.flag = !s.flag
          const l: string = s.label            // derived value, PRECISE string
          const w: string | null = s.ws        // derived value, string | null
          void l; void w
        },
        bad: (s) => {
          // @ts-expect-error count is a number signal — wrong value type
          s.count = 'not a number'
        },
      },
    })
  })
})
