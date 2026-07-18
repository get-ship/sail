<!--
  Reactivity (`/reactivity`) — the diamond-dependency glitch test, live.
  Change `a`; watch b, c, and d update; watch d recompute exactly once.
-->

<controller>
// Reactivity playground — the diamond-dependency glitch test.
//
//   a ─┬─▶ b = a × 2 ─┐
//      └─▶ c = a + 10 ─┴─▶ d = b + c
//
// When `a` changes, both b and c change, and both feed d. A naive reactive
// system recomputes d TWICE (once per changed input) — a "glitch", where d
// briefly holds an inconsistent value. SAIL's colored push-pull scheduler
// recomputes d exactly ONCE per `a` change. We prove it live: an effect counts
// d's recomputes; it should climb by exactly 1 each time you nudge `a`.
import { controller, signal, derived, effect, untrack, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

export default controller((_ctx: ControllerScope<Scope>) => {
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
  })
</controller>

<section data-rx="true">
  <header data-rx-head="true">
    <p role="doc-eyebrow">reactivity · the signal graph</p>
    <h1>The diamond, with no glitch.</h1>
    <p role="doc-lede">
      Change <code>a</code>. It feeds both <code>b</code> and <code>c</code>, and
      both feed <code>d</code>. SAIL recomputes <code>d</code> exactly once per
      change — never the double-fire a naive reactive system does.
    </p>
  </header>

  <div data-graph="true">
    <article data-node="signal" data-tier="source">
      <div data-node-head="true">
        <span data-tone="signal"></span><span data-name="true">a</span>
        <span data-kind="true">signal · source</span>
      </div>
      <div data-val="true">{a()}</div>
      <div data-controls="true">
        <button variant="default" type="button" on:click={() => dec()}>− 1</button>
        <button variant="solid" type="button" on:click={() => inc()}>+ 1</button>
        <button variant="ghost" type="button" on:click={() => resetValue()}>set 1</button>
      </div>
    </article>

    <p data-edge="true">feeds ↓</p>

    <div data-pair="true">
      <article data-node="derived">
        <div data-node-head="true"><span data-tone="derived"></span><span data-name="true">b</span><span data-kind="true">a × 2</span></div>
        <div data-val="true">{b()}</div>
      </article>
      <article data-node="derived">
        <div data-node-head="true"><span data-tone="derived"></span><span data-name="true">c</span><span data-kind="true">a + 10</span></div>
        <div data-val="true">{c()}</div>
      </article>
    </div>

    <p data-edge="true">join ↓</p>

    <article data-node="derived-join">
      <div data-node-head="true"><span data-tone="derived"></span><span data-name="true">d</span><span data-kind="true">b + c</span></div>
      <div>{d()}</div>
    </article>
  </div>

  <aside>
    <div>
      <div><span>you changed a</span><span>{aChanges()}×</span></div>
      <div><span>d recomputed</span><span>{dRecomputes()}×</span></div>
    </div>
    <p>
      One recompute per change (plus one at first paint). A glitchy system would
      <em>double</em> that — <code>d</code> would briefly equal
      <code>b_new + c_old</code>. SAIL's colored push-pull scheduler never lets
      that intermediate value exist.
    </p>
  </aside>

  <div role="navigation">
    <a href="/layouts">← Layouts</a>
    <a href="/data">The data seam →</a>
  </div>
</section>
