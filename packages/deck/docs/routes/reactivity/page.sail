<!--
  Live diamond-dependency playground: change a; b and c update; d recomputes once.
-->

<controller>
// Diamond graph: a → b, a → c, (b,c) → d. Effect counts d recomputes (should +1 per a change).
import { controller, signal, derived, effect, untrack, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

export default controller((_ctx: ControllerScope<Scope>) => {
    const a = signal(1)
    const b = derived(() => a() * 2)
    const c = derived(() => a() + 10)
    const d = derived(() => b() + c())

    const dRecomputes = signal(0)
    const aChanges = signal(0)

    // Track d; write the counter untracked so the effect does not re-run on its own output.
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
      // Reset a to 1 without clearing recompute counters (ratio stays as evidence).
      resetValue: (): void => nudge(1 - a()),
    }
  })
</controller>

<scope>
  style page = { maxW(breakpoint-tablet).mx(auto).padY(5) }
  style head = { mb(5) }
  style eyebrow = { mb(2).fg(accent).fontFamily(mono).fontSize(xs).tracking(wide).textCase(uppercase) }
  style lead = { maxW(panel-max-w-md).fg(muted-foreground).leading(relaxed) }
  style graph = { w(full).mb(5) }
  style node = { w(full).maxW(panel-max-w-md).padX(5).padY(4).mx(auto).bg(surface-1).borderWidth(1).border(border).radius(lg).textAlign(center) }
  style nodeHead = { mb(1) }
  style nodeName = { fg(foreground).fontFamily(mono).fontWeight(bold).fontSize(sm) }
  style nodeKind = { fg(muted-foreground).fontFamily(mono).fontSize(xs) }
  style val = { my(2).fg(foreground).fontFamily(mono).fontWeight(bold).fontSize(2xl).leading(none) }
  style valJoin = { my(2).fg(accent).fontFamily(mono).fontWeight(bold).fontSize(3xl).leading(none) }
  style controls = { mt(3) }
  style edge = { my(2).fg(muted-foreground).fontFamily(mono).fontSize(xs).tracking(wide).textAlign(center) }
  style pair = { w(full).maxW(panel-max-w-md).mx(auto).borderWidth(1).border(border).radius(lg).overflow(hidden) }
  style pairNode = { padX(5).padY(4).grow(1).borderRWidth(1).borderR(border).textAlign(center) }
  style pairNodeLast = { padX(5).padY(4).grow(1).textAlign(center) }
  style proof = { pad(5).mb(5).borderWidth(1).border(border).radius(lg).bg(surface-2) }
  style proofStats = { mb(3) }
  style stat = { minW(24) }
  style statK = { fg(muted-foreground).fontSize(xs).textCase(uppercase).tracking(wide) }
  style statV = { fontFamily(mono).fontSize(xl).fontWeight(bold) }
  style statAccent = { fg(accent).fontFamily(mono).fontSize(xl).fontWeight(bold) }
  style proofNote = { maxW(panel-max-w-md).fg(muted-foreground).fontSize(sm).leading(relaxed) }
  style footNav = { padT(4).borderTWidth(1).borderT(border) }
  style toneDot = { w(2).h(2).shrink(0).radius(full).bg(accent) }
  style toneDotMuted = { w(2).h(2).shrink(0).radius(full).bg(muted-foreground) }
</scope>

<section style(page)>
  <header style(head)>
    <p role="doc-eyebrow" style(eyebrow)>reactivity · the signal graph</p>
    <h1>The diamond, with no glitch.</h1>
    <p role="doc-lede" style(lead)>
      Change <code>a</code>. It feeds both <code>b</code> and <code>c</code>, and
      both feed <code>d</code>. SAIL recomputes <code>d</code> exactly once per
      change — never the double-fire a naive reactive system does.
    </p>
  </header>

  <column gap="2" align="center" style(graph)>
    <article style(node)>
      <row gap="2" align="center" justify="center" style(nodeHead)>
        <span style(toneDot) aria-hidden="true"></span>
        <span style(nodeName)>a</span>
        <span style(nodeKind)>signal · source</span>
      </row>
      <div style(val)>{a()}</div>
      <row gap="2" align="center" justify="center" style(controls)>
        <button variant="default" type="button" on:click={() => dec()}>− 1</button>
        <button variant="solid" type="button" on:click={() => inc()}>+ 1</button>
        <button variant="ghost" type="button" on:click={() => resetValue()}>set 1</button>
      </row>
    </article>

    <p style(edge)>feeds ↓</p>

    <row style(pair)>
      <article style(pairNode)>
        <row gap="2" align="center" justify="center" style(nodeHead)>
          <span style(toneDotMuted) aria-hidden="true"></span>
          <span style(nodeName)>b</span>
          <span style(nodeKind)>a × 2</span>
        </row>
        <div style(val)>{b()}</div>
      </article>
      <article style(pairNodeLast)>
        <row gap="2" align="center" justify="center" style(nodeHead)>
          <span style(toneDotMuted) aria-hidden="true"></span>
          <span style(nodeName)>c</span>
          <span style(nodeKind)>a + 10</span>
        </row>
        <div style(val)>{c()}</div>
      </article>
    </row>

    <p style(edge)>join ↓</p>

    <article style(node)>
      <row gap="2" align="center" justify="center" style(nodeHead)>
        <span style(toneDot) aria-hidden="true"></span>
        <span style(nodeName)>d</span>
        <span style(nodeKind)>b + c</span>
      </row>
      <div style(valJoin)>{d()}</div>
    </article>
  </column>

  <aside style(proof)>
    <row gap="6" style(proofStats)>
      <column gap="1" style(stat)>
        <span style(statK)>you changed a</span>
        <span style(statV)>{aChanges()}×</span>
      </column>
      <column gap="1" style(stat)>
        <span style(statK)>d recomputed</span>
        <span style(statAccent)>{dRecomputes()}×</span>
      </column>
    </row>
    <p style(proofNote)>
      One recompute per change (plus one at first paint). A glitchy system would
      <em>double</em> that — <code>d</code> would briefly equal
      <code>b_new + c_old</code>. The SAIL colored push-pull scheduler never lets
      that intermediate value exist.
    </p>
  </aside>

  <row justify="between" gap="3" style(footNav)>
    <a href="/layouts">← Layouts</a>
    <a href="/data">The data seam →</a>
  </row>
</section>
