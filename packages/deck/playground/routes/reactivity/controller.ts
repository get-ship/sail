// Reactivity playground — the diamond-dependency glitch test.
//
//   a ─┬─▶ b = a × 2 ─┐
//      └─▶ c = a + 10 ─┴─▶ d = b + c
//
// When `a` changes, both b and c change, and both feed d. A naive reactive
// system recomputes d TWICE (once per changed input) — a "glitch", where d
// briefly holds an inconsistent value. SUI's colored push-pull scheduler
// recomputes d exactly ONCE per `a` change. We prove it live: an effect counts
// d's recomputes; it should climb by exactly 1 each time you nudge `a`.
import { controller, signal, derived, effect, untrack, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const a = signal(1)
    const b = derived(() => a() * 2)
    const c = derived(() => a() + 10)
    const d = derived(() => b() + c())

    const dRecomputes = signal(0)
    const aChanges = signal(0)

    // Count d's actual recomputes. Track `d`; write the counter untracked so the
    // effect never depends on its own output (no loop).
    effect(() => {
      d()
      dRecomputes.set(untrack(() => dRecomputes()) + 1)
    })

    const nudge = (delta: number): void => {
      a.set(a() + delta)
      aChanges.set(untrack(() => aChanges()) + 1)
    }

    return {
      a,
      b,
      c,
      d,
      dRecomputes,
      aChanges,
      inc: (): void => nudge(1),
      dec: (): void => nudge(-1),
      // Reset `a` but KEEP the counters, so the "1 recompute per change" ratio
      // stays visible as evidence.
      resetValue: (): void => nudge(1 - a()),
    }
  },
})
