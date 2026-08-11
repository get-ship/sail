<!--
  /docs/concepts/animation — chapter 6: say what animation IS first; hero is
  tiles (no clipping text); full expression leads and presets are one
  paragraph; scroll/loop prose in plain English; choreography demo full-width
  with the code below it; Under the hood answers where Sail ends and the
  embedded Motion library begins. reveal is a Style animate trigger (not an
  HTML attr): animate: reveal({…}) — progress/view timeline with IO fallback.
-->

<scope>
  style page = { box: maxW(breakpoint-tablet).mx(auto) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style lead = { paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style pressBtn = { animate: press({ scale: 'press' }) }
  style tileRow = { box: minH(9) }
  style tile = { box: w(9).h(9), paint: radius(md).bg(primary), animate: enter({ translate-y: 'spring:snappy', stagger: '0.07s' }) }
  style tileDim = { box: w(9).h(9), paint: radius(md).bg(primary).opacity(75), animate: enter({ translate-y: 'spring:snappy', stagger: '0.07s' }) }
  style demoPanel = { box: pad(6).minH(8), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style demoPanelTall = { box: pad(6).minH(11), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style enterCard = { box: pad(4), animate: enter({ opacity: 'fade', scale: 'spring:snappy' }).exit({ opacity: 'fade-out' }) }
  style loopTileFast = { box: w(8).h(8), paint: radius(md).bg(primary), animate: loop({ translate-x: { from: 0, to: '80px' }, duration: 500 }) }
  style loopTileSlow = { box: w(8).h(8), paint: radius(md).bg(primary).opacity(75), animate: loop({ translate-x: { from: 0, to: '80px' }, duration: 2500 }) }
  style revealCard = { box: pad(6), animate: reveal({ opacity: 'fade', translate-y: 'spring:gentle' }) }
  style revealSpring = { box: pad(6), animate: reveal({ scale: 'spring:bouncy', opacity: 'fade' }) }
  style ambientDot = { box: w(4).h(4), paint: radius(full).bg(primary), animate: loop({ translate-y: 'spring:gentle', stagger: '0.15s' }) }
  style ambientDotDim = { box: w(4).h(4), paint: radius(full).bg(primary).opacity(75), animate: loop({ translate-y: 'spring:gentle', stagger: '0.15s' }) }
  style spinTile = { box: w(8).h(8), paint: radius(md).bg(primary), animate: loop({ rotate: 'spin' }) }
  style grow1 = { box: grow(1) }
  style scrub = { box: grow(1).minW(0) }
  style scrubVal = { box: w(8), paint: fg(muted-foreground), text: textAlign(end).fontFamily(mono).fontSize(sm) }
  style timelineTile = { box: shrink(0).w(9).h(9), paint: bg(primary).radius(md) }
  style focusBtn = { animate: focus-visible({ scale: 'spring:snappy' }) }
  style footNote = { box: mt(4), paint: fg(muted-foreground), text: fontSize(sm) }
  style footNav = { box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) }
</scope>

<controller>
// Replay clears the hand and re-deals after exits finish.

import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from '../../../../src/compositions/sail-language-animation-snippets'

export default controller((_ctx: ControllerScope<Scope>) => {
  const open = signal(true)
  const dealt = signal(true)
  const redeal = debounce(() => dealt.set(true), 450)
  const replay = (): void => {
    dealt.set(false)
    redeal()
  }

  // Slider owns progress 0…1; timeline() yields one signal per channel.
  const scrubText = signal('0')
  const p = (): number => Number(scrubText()) / 100
  const { tileX, tileRotate, tileScale } = timeline(
    p,
    {
      // Hold = same value at a later offset (not arithmetic).
      tileX: [[0, '0px'], [0.12, '0px'], [0.34, '120px'], [0.44, '120px'],
             [0.54, '120px'], [0.64, '120px'], [1, '0px']],
      tileRotate: [[0, '0deg'], [0.12, '0deg'], [0.34, '15deg'], [0.44, '15deg'],
                  [0.54, '15deg'], [0.64, '15deg'], [1, '0deg']],
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

<column gap="5" style(page)>
  <p style(eyebrow)>The language</p>
  <h1>Animation</h1>
  <p style(lead)>
    Animation is declared on the element via the Style <strong>animate
    channel</strong>. <code>style&#40;&#123; animate: enter(&#123;…&#125;) &#125;&#41;</code> /
    <code>exit</code> / <code>loop</code> / <code>reveal</code> name a
    <strong>trigger</strong> — <code>enter</code>, <code>exit</code>,
    <code>reveal</code>, <code>press</code>, <code>loop</code> — and a
    <strong>config</strong>: which properties move, how far, how fast. When
    something else owns the clock — a slider, the scroll position —
    <code>timeline()</code> turns keyframes into signals. This page is both,
    in that order.
  </p>

  <row gap="3" align="center">
    <button on:click={replay} style(pressBtn)>Replay</button>
    <p style(mutedSm)>
      Five tiles, one <code>style&#40;&#123; animate: enter(&#123;…&#125;) &#125;&#41;</code> each;
      <code>stagger</code> is the delay between siblings.
    </p>
  </row>
  <if {dealt()}>
    <row gap="3" align="center" style(tileRow)>
      <div style(tile)></div>
      <div style(tileDim)></div>
      <div style(tile)></div>
      <div style(tileDim)></div>
      <div style(tile)></div>
    </row>
  </if>
  <code-block code={code.stagger}></code-block>

  <h2>Enter and exit</h2>
  <p>
    <code>enter</code> plays when the element joins the page;
    <code>exit</code> plays when it leaves. Exit is the one that needs the
    framework’s help: remove a node yourself and its animation has nowhere
    to play, so the runtime keeps the element mounted until the exit
    finishes.
  </p>
  <column gap="3" style(demoPanel)>
    <row>
      <button on:click={() => open.set(!open())} style(pressBtn)>

        {open() ? 'Remove' : 'Add'} the panel
      </button>
    </row>
    <if {open()}>
      <card style(enterCard)>
        <strong>I animate in, and I animate out.</strong>
        <p style(mutedSm)>Watch me leave before I go.</p>
      </card>
    </if>
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
  <row gap="5" align="center" style(tileRow)>
    <div testid="inline-demo" style(loopTileFast)></div>
    <div style(loopTileSlow)></div>
  </row>
  <code-block code={code.express}></code-block>
  <p>
    Common shapes have names — <code>'fade'</code>,
    <code>'spring:snappy'</code>, <code>'pulse'</code> — and a name is just
    a config you didn’t have to write. Start from one and override any
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
    <card style(revealCard)>
      <h3>This one fades up as it enters the viewport.</h3>
    </card>
    <card style(revealSpring)>
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
  <row gap="3" align="center" id="loop-row" style(tileRow)>
    <div testid="ambient-dot" style(ambientDot)></div>
    <div style(ambientDotDim)></div>
    <div style(ambientDot)></div>
    <div style(ambientDotDim)></div>
    <row style(grow1)></row>
    <div testid="spin-demo" style(spinTile)></div>
  </row>
  <code-block code={code.loop}></code-block>

  <h2>Gestures</h2>
  <p>
    <code>press</code> reacts to the press-and-release gesture;
    <code>focus-visible</code> to the keyboard arriving. Hover feedback is
    not animation — it’s a <code>hover:</code> atom in the
    <a href="/docs/concepts/styling">styling chapter</a>.
  </p>
  <row gap="3" align="center">
    <button style(pressBtn)>press me</button>
    <button style(focusBtn)>tab to me</button>
  </row>
  <code-block code={code.gestures}></code-block>

  <h2>When something else owns the clock</h2>
  <p>
    Everything above starts, plays, and finishes on its own. Some animation
    instead has to <em>follow</em> something — this slider. Drag it: the
    tile moves through a seven-stop sequence, holds where the keyframes
    hold, and pops in place between 54 and 64.
  </p>
  <column gap="4" style(demoPanelTall)>
    <row align="center" gap="3">
      <input type="range" min="0" max="100" step="1" bind:value={scrubText} testid="timeline-scrub" style(scrub)>
      <span style(scrubVal)>{scrubText()}</span>
    </row>
    <row align="center" style(tileRow)>
      <!-- Plain div (not card): card host paints surface, so primary fill would not win. -->
      <div testid="timeline-tile" style(timelineTile, { paint: translateX(bind(tileX)).rotate(bind(tileRotate)).scale(bind(tileScale)) })></div>
    </row>
  </column>
  <p>
    The program behind it is <code>timeline()</code>: it takes a progress
    reader — here, the slider’s value scaled to 0…1 — and a set of
    keyframes per property, and returns one signal per property. Those
    signals reach Style the way any signal does, through
    <code>bind&#40;name&#41;</code> on a setter
    (<code>style&#40;&#123; paint: translateX(bind(tileX)) &#125;&#41;</code>):
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
    declarations onto it — Style motion methods become Motion
    animations, springs use Motion’s spring generator. What Sail adds is the
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
      <p style(mutedSm)>enter · exit · reveal · scroll · press · focus · focus-visible · loop</p>
    </column>
    <column gap="1">
      <label>Properties</label>
      <p style(mutedSm)>opacity · scale · scale-x · scale-y · translate-x · translate-y · rotate</p>
    </column>
    <column gap="1">
      <label>Springs</label>
      <p style(mutedSm)>spring:snappy · spring:gentle · spring:bouncy · spring:deliberate</p>
    </column>
    <column gap="1">
      <label>Tweens</label>
      <p style(mutedSm)>fade · fade-out · press · release · pulse · spin</p>
    </column>
  </grid>
  <p style(mutedSm)>
    Config fields: <code>duration</code> · <code>delay</code> ·
    <code>easing</code> · <code>from</code>/<code>to</code> ·
    <code>stagger</code>. Every name above is compiler-checked; every value
    is yours.
  </p>

  <p style(footNote)>
    For the full surface: <a href="/docs/reference/motion">reference → Motion</a>.
  </p>

  <row justify="between" gap="2" style(footNav)>
    <a variant="default" href="/docs/concepts/layout">← Layout</a>
    <a variant="solid" tone="primary" href="/docs/concepts/actions">Actions →</a>
  </row>
</column>
