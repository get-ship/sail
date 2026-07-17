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

  it('canonical form: setup gets a PRECISE, cast-free ViewScope inferred from Tpl', () => {
    // `controller((ctx: ViewScope<Tpl>) => ({…}))` infers the template
    // `Ctx` from the setup function's param annotation — the compiler-
    // generated `ViewScope<Tpl>` the author imports. Signals it creates
    // itself (`signal`/`derived`) are read/written directly through the
    // `Signal<T>` API — there is no separate writable-proxy type for
    // handlers; a handler returned from setup closes over the same
    // signal and calls `.set()` on it, same as setup itself would.
    interface Tpl { /* template-declared <data>/<state> only */ }
    controller((_ctx: ViewScope<Tpl>) => {
      const count = signal(0)
      const flag = signal(false)
      const label = derived(() => 'hello')
      const ws = derived(() => 'a' as string | null)
      return {
        count,
        flag,
        label,
        ws,
        bump: () => {
          count.set(count() + 1)
          flag.set(!flag())
          const l: string = label()   // derived value, PRECISE string
          const w: string | null = ws()
          void l; void w
        },
        bad: () => {
          // @ts-expect-error count is a number signal — wrong value type
          count.set('not a number')
        },
      }
    })
  })
})
