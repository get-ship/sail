<!--
  Animation chapter — HEAD content. Style lives here (L2) so the route
  does not spend it. Tiles are play(rise|pulse|bob|spin), not <spinner>.
-->
<scope>
  style pressBtn = { animate: { press: play(press) } }
  style tile = { size(9).radius(md).bg(primary), animate: { enter: play(rise) } }
  style tileDim = { size(9).radius(md).bg(accent), animate: { enter: play(rise) } }
  style enterCard = { animate: { enter: play(fade-scale-in), exit: play(fade) } }
  style loopTileFast = { size(8).radius(md).bg(primary), animate: { loop: play(pulse, 500) } }
  style loopTileSlow = { size(8).radius(md).bg(accent), animate: { loop: play(pulse, 2500) } }
  style revealCard = { overflow(visible), animate: { reveal: play(slide-up) } }
  style revealSpring = { overflow(visible), animate: { reveal: play(fade-scale-in) } }
  style ambientDot = { size(4).radius(full).bg(primary), animate: { loop: play(bob) } }
  style ambientDotDim = { size(4).radius(full).bg(accent), animate: { loop: play(bob) } }
  style spinTile = { size(8).radius(md).bg(primary), animate: { loop: play(spin) } }
  style grow1 = { size(grow) }
  style scrub = { size(grow) }
  style timelineTile = { size(9).bg(primary).radius(md) }
  style focusBtn = { animate: { focus-visible: play(pulse) } }
  style muted = { fg(muted).type(body-sm) }
  style eyebrow = { fg(accent).type(label) }
</scope>

<controller>
import { code } from '../compositions/sail-language-animation-snippets'

export default controller(() => {
  const open = signal(true)
  const dealt = signal(true)
  const redeal = debounce(() => dealt.set(true), 450)
  const replay = (): void => {
    dealt.set(false)
    redeal()
  }
  const toggleOpen = (): void => {
    open.set(!open())
  }

  const scrubText = signal('0')
  const p = (): number => Number(scrubText()) / 100
  const { tileX, tileRotate, tileScale } = timeline(
    p,
    {
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
    toggleOpen,
    scrubText,
    tileX,
    tileRotate,
    tileScale,
  }
})
</controller>

<column gap="5">
  <p style(eyebrow)>The language</p>
  <h1>Animation</h1>
  <p>
    Animation is declared on the element via the Style <strong>animate
    channel</strong>. <code>style&#40;&#123; animate: &#123; enter: play(fade) &#125; &#125;&#41;</code> /
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
    <p style(muted)>
      Five tiles, one <code>style&#40;&#123; animate: &#123; enter: play(rise) &#125; &#125;&#41;</code> each;
      <code>stagger</code> is the delay between siblings.
    </p>
  </row>
  <if {dealt()}>
    <row gap="3" align="center">
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
  <card>
    <column gap="3">
      <row>
        <button on:click={toggleOpen} style(pressBtn)>
          {open() ? 'Remove' : 'Add'} the panel
        </button>
      </row>
      <if {open()}>
        <card style(enterCard)>
          <strong>I animate in, and I animate out.</strong>
          <p style(muted)>Watch me leave before I go.</p>
        </card>
      </if>
    </column>
  </card>
  <code-block code={code.lifecycle}></code-block>

  <h2>Write the animation you mean</h2>
  <p>
    Every field of a config takes a value directly:
    <code>duration</code> and <code>delay</code> in milliseconds,
    <code>easing</code> as a keyword or a cubic-bezier, and
    <code>from</code>/<code>to</code> per property. The two tiles below run
    the same loop — one at 500&nbsp;ms, one at 2500&nbsp;ms:
  </p>
  <row gap="5" align="center">
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
  <row gap="3" align="center" id="loop-row">
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
  <card>
    <column gap="4">
      <row align="center" gap="3">
        <slider min="0" max="100" step="1" bind:value={scrubText} testid="timeline-scrub" style(scrub) />
        <span>{scrubText()}</span>
      </row>
      <row align="center">
        <div testid="timeline-tile" style(timelineTile, { translateX(bind(tileX)).rotate(bind(tileRotate)).scale(bind(tileScale)) })></div>
      </row>
    </column>
  </card>
  <p>
    The program behind it is <code>timeline()</code>: it takes a progress
    reader — here, the slider’s value scaled to 0…1 — and a set of
    keyframes per property, and returns one signal per property. Those
    signals reach Style the way any signal does, through
    <code>bind&#40;name&#41;</code> on a setter
    (<code>style&#40;&#123; translateX(bind(tileX)) &#125;&#41;</code>):
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
      <p style(muted)>enter · exit · reveal · scroll · press · focus · focus-visible · loop</p>
    </column>
    <column gap="1">
      <label>Properties</label>
      <p style(muted)>opacity · scale · scale-x · scale-y · translate-x · translate-y · rotate</p>
    </column>
    <column gap="1">
      <label>Springs</label>
      <p style(muted)>spring:snappy · spring:gentle · spring:bouncy · spring:deliberate</p>
    </column>
    <column gap="1">
      <label>Tweens</label>
      <p style(muted)>fade · fade-out · fade-scale-in · slide-up · rise · press · release · pulse · spin</p>
    </column>
  </grid>
  <p style(muted)>
    Config fields: <code>duration</code> · <code>delay</code> ·
    <code>easing</code> · <code>from</code>/<code>to</code> ·
    <code>stagger</code>. Every name above is compiler-checked; every value
    is yours.
  </p>

  <p style(muted)>
    For the full surface: <a href="/docs/reference/motion">reference → Motion</a>.
  </p>

  <separator />
  <row justify="between" gap="2">
    <a variant="default" href="/docs/concepts/layout">← Layout</a>
    <a variant="solid" tone="primary" href="/docs/concepts/actions">Actions →</a>
  </row>
</column>
