<!--
  /sail/language/motion — chapter 6, rebuilt per owner review: say what motion
  IS first; hero is tiles (no clipping text); full expression leads and
  presets are one paragraph; scroll/loop prose in plain English; choreography
  demo full-width with the code below it; compositor talk and the 30 kB jab
  cut; Under the hood answers where Sail ends and the embedded Motion library
  begins. The motion:reveal demos are correct as written — the reveal trigger
  currently attaches nothing at runtime (regression, job/motion-view-regression
  dispatched with this page as the gate).
-->

<controller>
// /motion — the page deals its own cards. `replay` clears the hand and
// re-deals after the exits finish, so the stagger arrival can be watched
// again without a reload. No interval: one deal on arrival, replay on demand.

import { controller, signal, timeline, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  const open = signal(true)
  const dealt = signal(true)
  const replay = (): void => {
    dealt.set(false)
    setTimeout(() => dealt.set(true), 450)
  }

  // The choreography lane, live. The slider owns the clock; `timeline()` turns
  // motion.dev's own 7-stop tile keyframes into one signal per channel. Swap
  // `p` for `clock(3600, { loop: true })` and the demo runs itself — nothing
  // else on this page would change.
  const scrubText = signal('0')
  const p = (): number => Number(scrubText()) / 100
  const { tileX, tileRotate, tileScale } = timeline(
    p,
    {
      // Translate settles at .34 and HOLDS through .44 — a hold is the same
      // VALUE repeated at a later offset, never arithmetic.
      tileX: [[0, '0px'], [0.12, '0px'], [0.34, '120px'], [0.44, '120px'],
             [0.54, '120px'], [0.64, '120px'], [1, '0px']],
      tileRotate: [[0, '0deg'], [0.12, '0deg'], [0.34, '15deg'], [0.44, '15deg'],
                  [0.54, '15deg'], [0.64, '15deg'], [1, '0deg']],
      // …while the scale pops on its own, .54 → .64.
      tileScale: [[0.44, 1], [0.54, 1.75], [0.64, 1]],
    },
    { easing: [0.6, 0, 0.2, 1] },
  )

  return {
    code,
    open,
    dealt,
    replay,
    hand: signal(['A', 'K', 'Q', 'J', '10']),
    scrubText,
    tileX,
    tileRotate,
    tileScale,
  }
})
</controller>
<column sx={style.maxW(breakpoint-tablet).mx(auto)} gap="5">
  <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)}>The language</p>
  <h1>Motion</h1>
  <p sx={style.fontSize(lg).fg(muted-foreground).leading(relaxed)}>
    Animation is declared on the element. <code>motion:</code> names a
    <strong>trigger</strong> — <code>enter</code>, <code>exit</code>,
    <code>reveal</code>, <code>press</code>, <code>loop</code> — and a
    <strong>config</strong>: which properties move, how far, how fast. When
    something else owns the clock — a slider, the scroll position —
    <code>timeline()</code> turns keyframes into signals. This page is both,
    in that order.
  </p>

  <row gap="3" align="center">
    <button on:click={replay} motion:press={{ scale: 'press' }}>Replay</button>
    <p sx={style.fontSize(sm).fg(muted-foreground)}>
      Five tiles, one <code>motion:enter</code> each; <code>stagger</code>
      is the delay between siblings.
    </p>
  </row>
  {#if dealt()}
    <row gap="3" align="center" sx={style.minH(9)}>
      <div sx={style.w(9).h(9).radius(md).bg(primary)} motion:enter={{ translate-y: 'spring:snappy', stagger: '0.07s' }}></div>
      <div sx={style.w(9).h(9).radius(md).bg(primary).opacity(75)} motion:enter={{ translate-y: 'spring:snappy', stagger: '0.07s' }}></div>
      <div sx={style.w(9).h(9).radius(md).bg(primary)} motion:enter={{ translate-y: 'spring:snappy', stagger: '0.07s' }}></div>
      <div sx={style.w(9).h(9).radius(md).bg(primary).opacity(75)} motion:enter={{ translate-y: 'spring:snappy', stagger: '0.07s' }}></div>
      <div sx={style.w(9).h(9).radius(md).bg(primary)} motion:enter={{ translate-y: 'spring:snappy', stagger: '0.07s' }}></div>
    </row>
  {/if}
  <code-block code={code.stagger}></code-block>

  <h2>Enter and exit</h2>
  <p>
    <code>enter</code> plays when the element joins the page;
    <code>exit</code> plays when it leaves. Exit is the one that needs the
    framework's help: remove a node yourself and its animation has nowhere
    to play, so the runtime keeps the element mounted until the exit
    finishes.
  </p>
  <column sx={style.pad(6).borderWidth(1).border(border).radius(lg).minH(8).bg(surface-1)} gap="3">
    <row>
      <button on:click={() => open.set(!open())} motion:press={{ scale: 'press' }}>
        {open() ? 'Remove' : 'Add'} the panel
      </button>
    </row>
    {#if open()}
      <card
        motion:enter={{ opacity: 'fade', scale: 'spring:snappy' }}
        motion:exit={{ opacity: 'fade-out' }}
        sx={style.pad(4)}
      >
        <strong>I animate in, and I animate out.</strong>
        <p sx={style.fontSize(sm).fg(muted-foreground)}>Watch me leave before I go.</p>
      </card>
    {/if}
  </column>
  <code-block code={code.lifecycle}></code-block>

  <h2>Write the animation you mean</h2>
  <p>
    Every field of a config takes a value directly:
    <code>duration</code> and <code>delay</code> in milliseconds,
    <code>easing</code> as a keyword or a cubic-bezier, and
    <code>from</code>/<code>to</code> per property. The two tiles below run
    the same loop — one at 500&nbsp;ms, one at 2500&nbsp;ms:
  </p>
  <row gap="5" align="center" sx={style.minH(9)}>
    <div sx={style.w(8).h(8).radius(md).bg(primary)} motion:loop={{ translate-x: { from: 0, to: '80px' }, duration: 500 }}></div>
    <div sx={style.w(8).h(8).radius(md).bg(primary).opacity(75)} motion:loop={{ translate-x: { from: 0, to: '80px' }, duration: 2500 }}></div>
  </row>
  <code-block code={code.express}></code-block>
  <p>
    Common shapes have names — <code>'fade'</code>,
    <code>'spring:snappy'</code>, <code>'pulse'</code> — and a name is just
    a config you didn't have to write. Start from one and override any
    field, or skip them entirely. One spelling rule covers every value:
    quoted is a literal, unquoted is code — which is how a signal goes
    anywhere a number can.
  </p>

  <h2>On scroll — <code>reveal</code></h2>
  <p>
    <code>reveal</code> plays the animation when the element scrolls into the
    viewport. Same configs as everything else:
  </p>
  <column gap="4">
    <card motion:reveal={{ opacity: 'fade', translate-y: 'spring:gentle' }} sx={style.pad(6)}>
      <h3>This one fades up as it enters the viewport.</h3>
    </card>
    <card motion:reveal={{ scale: 'spring:bouncy', opacity: 'fade' }} sx={style.pad(6)}>
      <h3>And this one springs in.</h3>
    </card>
  </column>
  <code-block code={code.reveal}></code-block>

  <h2>Forever — <code>loop</code></h2>
  <p>
    <code>loop</code> repeats until the element leaves — for the decorative
    layer: pulsing dots, a spinner, a status light. <code>stagger</code>
    offsets siblings so a row reads as a wave. A user whose system asks for
    reduced motion gets none of this, automatically.
  </p>
  <row gap="3" align="center" sx={style.minH(9)}>
    <div sx={style.w(4).h(4).radius(full).bg(primary)} motion:loop={{ translate-y: 'spring:gentle', stagger: '0.15s' }}></div>
    <div sx={style.w(4).h(4).radius(full).bg(primary).opacity(75)} motion:loop={{ translate-y: 'spring:gentle', stagger: '0.15s' }}></div>
    <div sx={style.w(4).h(4).radius(full).bg(primary)} motion:loop={{ translate-y: 'spring:gentle', stagger: '0.15s' }}></div>
    <div sx={style.w(4).h(4).radius(full).bg(primary).opacity(75)} motion:loop={{ translate-y: 'spring:gentle', stagger: '0.15s' }}></div>
    <row sx={style.grow(1)}></row>
    <div sx={style.w(8).h(8).radius(md).bg(primary)} motion:loop={{ rotate: 'spin' }}></div>
  </row>
  <code-block code={code.loop}></code-block>

  <h2>Gestures</h2>
  <p>
    <code>press</code> reacts to the press-and-release gesture;
    <code>focus-visible</code> to the keyboard arriving. Hover feedback is
    not animation — it's a <code>hover:</code> atom in the
    <a href="/sail/language/styling">styling chapter</a>.
  </p>
  <row gap="3" align="center">
    <button motion:press={{ scale: 'press' }}>press me</button>
    <button motion:focus-visible={{ scale: 'spring:snappy' }}>tab to me</button>
  </row>
  <code-block code={code.gestures}></code-block>

  <h2>When something else owns the clock</h2>
  <p>
    Everything above starts, plays, and finishes on its own. Some animation
    instead has to <em>follow</em> something — this slider. Drag it: the
    tile moves through a seven-stop sequence, holds where the keyframes
    hold, and pops in place between 54 and 64.
  </p>
  <column sx={style.pad(6).borderWidth(1).border(border).radius(lg).bg(surface-1).minH(11)} gap="4">
    <row align="center" gap="3">
      <input
        type="range"
        min="0"
        max="100"
        step="1"
        bind:value={scrubText}
        data-testid="timeline-scrub"
        sx={style.grow(1).minW(0)}
      />
      <span sx={style.w(8).textAlign(end).fontFamily(mono).fontSize(sm).fg(muted-foreground)}>{scrubText()}</span>
    </row>
    <row sx={style.minH(9)} align="center">
      <card
        data-testid="timeline-tile"
        sx={style.w(9).h(9).bg(primary).radius(md).translateX(bind(tileX)).rotate(bind(tileRotate)).scale(bind(tileScale))}
      ></card>
    </row>
  </column>
  <p>
    The program behind it is <code>timeline()</code>: it takes a progress
    reader — here, the slider's value scaled to 0…1 — and a set of
    keyframes per property, and returns one signal per property. Those
    signals reach Style the way any signal does, through
    <code>bind(name)</code> on a setter
    (<code>style.translateX(bind(tileX))</code>):
  </p>
  <code-block code={code.choreography}></code-block>
  <p>
    A hold is the same value repeated at a later offset. And because
    progress is just a signal, the slider is swappable for anything that
    produces one:
  </p>
  <code-block code={code.drivers}></code-block>

  <h2>Under the hood</h2>
  <p>
    Sail does not reimplement animation. The runtime embeds the
    <a href="https://motion.dev">Motion</a> library and compiles your
    declarations onto it — <code>motion:</code> attributes become Motion
    animations, springs use Motion's spring generator. What Sail adds is the
    declaration layer: the compiler checks every trigger, property, and
    preset name at build, so a typo is a compile error rather than an
    animation that silently never runs. In-editor autocomplete inside the
    config braces is not there yet; the vocabulary below is the whole
    surface in the meantime.
  </p>

  <h2>The vocabulary</h2>
  <grid cols="2" gap="4">
    <column gap="1">
      <label>Triggers</label>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>enter · exit · reveal · scroll · press · focus · focus-visible · loop</p>
    </column>
    <column gap="1">
      <label>Properties</label>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>opacity · scale · scale-x · scale-y · translate-x · translate-y · rotate</p>
    </column>
    <column gap="1">
      <label>Springs</label>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>spring:snappy · spring:gentle · spring:bouncy · spring:deliberate</p>
    </column>
    <column gap="1">
      <label>Tweens</label>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>fade · fade-out · press · release · pulse · spin</p>
    </column>
  </grid>
  <p sx={style.fontSize(sm).fg(muted-foreground)}>
    Config fields: <code>duration</code> · <code>delay</code> ·
    <code>easing</code> · <code>from</code>/<code>to</code> ·
    <code>stagger</code>. Every name above is compiler-checked; every value
    is yours.
  </p>

  <p sx={style.mt(4).fontSize(sm).fg(muted-foreground)}>
    For the full surface: <a href="/sail/reference/motion">reference → Motion</a>.
  </p>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)} justify="between" gap="2">
    <a variant="default" href="/sail/language/layout">← Layout</a>
    <a variant="solid" tone="primary" href="/sail/language/actions">Actions →</a>
  </row>
</column>
