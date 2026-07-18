<!--
  /motion — the page must MOVE, unprompted. The hand deals itself the moment
  you arrive; every section is the live thing beside the code that made it.

  Authored under the floor: no style blocks, no raw containers, no inline
  style attributes.
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

<column sx="max-w-breakpoint-tablet mx-auto" gap="5">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Components</p>
  <h1>Motion</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    One directive. <code>motion:</code> takes a <strong>trigger</strong> — when —
    and a <strong>config</strong> — what moves. No second animation API, no
    imperative escape hatch you are meant to graduate to. The cards below dealt
    themselves when you arrived.
  </p>

  <row gap="3" align="center">
    <button on:click={replay} motion:press={{ scale: 'press' }}>Deal again</button>
    <p sx="text-sm text-muted-foreground">
      <code>stagger</code> is the delay <em>between</em> siblings — a list
      arrives as a list, not a flash.
    </p>
  </row>

  {#if dealt()}
    <grid cols="5" gap="3">
      <card motion:enter={{ translate-y: 'spring:snappy', stagger: '0.07s' }} motion:exit={{ opacity: 'fade-out' }}
            motion:hover={{ scale: 'spring:snappy' }} sx="p-5">
        <h3>snappy</h3>
        <p sx="text-sm text-muted-foreground">translate-y</p>
      </card>
      <card motion:enter={{ opacity: 'fade', stagger: '0.07s' }} motion:exit={{ opacity: 'fade-out' }}
            motion:hover={{ translate-y: 'spring:gentle' }} sx="p-5">
        <h3>fade</h3>
        <p sx="text-sm text-muted-foreground">opacity only</p>
      </card>
      <card motion:enter={{ rotate: 'spring:bouncy', stagger: '0.07s' }} motion:exit={{ opacity: 'fade-out' }}
            motion:hover={{ rotate: 'spring:bouncy' }} sx="p-5">
        <h3>bouncy</h3>
        <p sx="text-sm text-muted-foreground">rotate</p>
      </card>
      <card motion:enter={{ translate-x: 'spring:deliberate', stagger: '0.07s' }} motion:exit={{ opacity: 'fade-out' }}
            motion:hover={{ translate-x: 'spring:deliberate' }} sx="p-5">
        <h3>deliberate</h3>
        <p sx="text-sm text-muted-foreground">translate-x</p>
      </card>
      <card motion:enter={{ scale: 'spring:gentle', stagger: '0.07s' }} motion:exit={{ opacity: 'fade-out' }}
            motion:hover={{ opacity: 'pulse' }} sx="p-5">
        <h3>gentle</h3>
        <p sx="text-sm text-muted-foreground">scale</p>
      </card>
    </grid>
  {/if}

  <code-block code={code.stagger}></code-block>

  <h2>Interaction</h2>
  <grid cols="2" gap="4">
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Live — hover, press, tab to it</p>
      <column sx="p-6 border border-border rounded-lg min-h-8 bg-surface-1" gap="3">
        <row gap="3">
          <card motion:hover={{ scale: 'spring:gentle' }} sx="p-4">
            <strong>lift</strong>
            <p sx="text-sm text-muted-foreground">hover · scale</p>
          </card>
          <card motion:hover={{ rotate: 'spring:bouncy' }} sx="p-4">
            <strong>tilt</strong>
            <p sx="text-sm text-muted-foreground">hover · rotate</p>
          </card>
          <card motion:hover={{ translate-x: 'spring:snappy' }} sx="p-4">
            <strong>nudge</strong>
            <p sx="text-sm text-muted-foreground">hover · translate-x</p>
          </card>
        </row>
        <row gap="3" align="center">
          <button motion:press={{ scale: 'press' }}>press</button>
          <button motion:press={{ rotate: 'spring:snappy' }}>press · rotate</button>
          <button motion:focus-visible={{ scale: 'spring:snappy' }}>tab to me</button>
          <badge tone="primary" motion:hover={{ scale: 'pulse' }}>hover · pulse</badge>
        </row>
      </column>
    </column>
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Source</p>
      <code-block code={code.interaction}></code-block>
    </column>
  </row>
  <p>
    The browser already knows when these happen. The only question a trigger
    leaves open is what it should look like — and the answer is a preset, so a
    typo is a compile error instead of something you ship.
  </p>

  <h2>Lifecycle</h2>
  <grid cols="2" gap="4">
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Live</p>
      <column sx="p-6 border border-border rounded-lg min-h-8 bg-surface-1" gap="3">
        <button on:click={() => open.set(!open())} motion:press={{ scale: 'press' }}>
          {open() ? 'Remove' : 'Add'} the panel
        </button>
        {#if open()}
          <card
            motion:enter={{ opacity: 'fade', scale: 'spring:snappy' }}
            motion:exit={{ opacity: 'fade-out' }}
            sx="p-4"
          >
            <strong>I animate in, and I animate out.</strong>
            <p sx="text-sm text-muted-foreground">
              The runtime holds me in the DOM until my exit finishes.
            </p>
          </card>
        {/if}
      </column>
    </column>
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Source</p>
      <code-block code={code.lifecycle}></code-block>
    </column>
  </row>
  <p>
    <code>exit</code> is the one a framework must be involved in: remove a node
    yourself and its exit animation has nowhere to play. The runtime keeps the
    element mounted until the animation finishes — watch the panel leave before
    it goes.
  </p>

  <h2>Timeline — scroll, not time</h2>
  <p>
    <code>view</code> and <code>scroll</code> are driven by scroll
    <em>position</em>. Where the platform supports scroll timelines the browser
    drives them on the compositor — <strong>no JavaScript runs while you
    scroll</strong>. Elsewhere they play once on reveal. The effect people
    install a 30&nbsp;kB library for is a directive with a preset.
  </p>
  <p sx="text-muted-foreground">Keep scrolling. Each of these arrives on its own.</p>

  <column gap="4">
    <card motion:view={{ opacity: 'fade', translate-y: 'spring:gentle' }} sx="p-6">
      <h3>This one fades up as it enters the viewport.</h3>
      <p sx="text-muted-foreground"><code>motion:view</code> — nothing in the frame loop.</p>
    </card>
    <card motion:view={{ opacity: 'fade', translate-x: 'spring:deliberate' }} sx="p-6">
      <h3>This one slides in, slowly.</h3>
      <p sx="text-muted-foreground">Same directive, different preset — <code>spring:deliberate</code>.</p>
    </card>
    <card motion:view={{ scale: 'spring:bouncy', opacity: 'fade' }} sx="p-6">
      <h3>And this one springs.</h3>
      <p sx="text-muted-foreground">Nine presets. Four springs, five tweens. That is the whole catalogue.</p>
    </card>
  </column>

  <code-block code={code.timeline}></code-block>

  <h2>Ambient — loop</h2>
  <p>
    <code>loop</code> is the idle life of a page: the baked animation
    alternates forever, <code>stagger</code> becomes a phase offset so
    siblings read as a wave, and <code>prefers-reduced-motion</code> switches
    it off entirely — ambient motion is decoration.
  </p>
  <row gap="3" align="center" sx="min-h-9">
    <div sx="w-4 h-4 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.15s' }}></div>
    <div sx="w-4 h-4 rounded-full bg-primary opacity-75" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.15s' }}></div>
    <div sx="w-4 h-4 rounded-full bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.15s' }}></div>
    <div sx="w-4 h-4 rounded-full bg-primary opacity-75" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.15s' }}></div>
  </row>
  <code-block code={code.loop}></code-block>

  <p sx="text-sm text-muted-foreground">continuous rotate — one direction, never ping-pongs</p>
  <row gap="3" align="center" sx="min-h-9">
    <div data-testid="spin-demo" sx="w-8 h-8 rounded-md bg-primary" motion:loop={{ rotate: 'spin' }}></div>
  </row>
  <code-block code={code.spin}></code-block>

  <h2>Choreography — when something else owns the clock</h2>
  <p>
    Everything above is fire-and-forget: you name a preset and the browser runs
    it. That is the right answer right up until the clock belongs to something
    else — a slider, a scroll position, a sequence that has to land on exact
    offsets and <em>stay there</em>. <code>motion:</code> cannot express that,
    and the honest alternative used to be hand-rolled
    <code>Math.min/Math.max</code> arithmetic in an effect.
  </p>
  <p>
    <code>timeline()</code> is that arithmetic as <strong>data</strong>. It
    takes a progress reader and returns one signal per track — which the style
    layer already knows how to read, because
    <code>(--tile-x)</code> is the same reflection a plain
    <code>signal(...)</code> gets. The tile below is motion.dev's own seven-stop
    choreography, keyframe for keyframe.
  </p>

  <grid cols="2" gap="4">
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Live — drag the slider</p>
      <column sx="p-6 border border-border rounded-lg bg-surface-1 min-h-11" gap="4">
        <row align="center" gap="3">
          <input
            type="range"
            min="0"
            max="100"
            step="1"
            bind:value={scrubText}
            data-testid="timeline-scrub"
            sx="grow-1 min-w-0"
          />
          <span sx="w-8 text-end font-mono text-sm text-muted-foreground">{scrubText()}</span>
        </row>
        <row sx="min-h-9" align="center">
          <card
            data-testid="timeline-tile"
            sx="w-9 h-9 bg-primary rounded-md translate-x-(--tile-x) rotate-(--tile-rotate) scale-(--tile-scale)"
          ></card>
        </row>
        <p sx="text-sm text-muted-foreground">
          Watch it settle at 34 and <strong>hold</strong> — then pop, in place,
          between 54 and 64. Three channels, one progress value.
        </p>
      </column>
    </column>
    <column>
      <p sx="font-mono text-xs uppercase tracking-wide text-muted-foreground mb-2">Source</p>
      <code-block code={code.choreography}></code-block>
    </column>
  </grid>

  <p>
    A hold is the same <em>value</em> repeated at a later offset — never a
    conditional, never a clamp. Per-segment easing follows the platform's own
    rule (a stop eases the segment it <em>starts</em>), so keyframes captured
    off a real page transfer verbatim.
  </p>

  <h3>Drivers</h3>
  <p>
    Progress is just a signal, so the thing producing it is interchangeable.
    That is why the slider above and a self-running animation are the same
    program with one line changed.
  </p>
  <code-block code={code.drivers}></code-block>

  <alert tone="info">
    <strong>Two lanes, one rule.</strong> If the browser can own the clock, let
    it — <code>motion:</code> runs on the compositor and costs the main thread
    nothing. If something else owns the clock, reach for
    <code>timeline()</code>. It writes custom properties every frame, which is
    real main-thread work, and it holds 60fps at sixty elements across three
    channels each — far past what a page actually choreographs at once.
  </alert>

  <h2>The vocabulary</h2>
  <p>
    Small on purpose, and enforced: an unknown trigger or preset is a compile
    error, not a silent no-op. A typo used to be something you could ship.
  </p>

  <grid cols="2" gap="4">
    <column gap="1">
      <label>Triggers</label>
      <p sx="text-sm text-muted-foreground">hover · press · focus · focus-visible · enter · exit · view · scroll · loop</p>
    </column>
    <column gap="1">
      <label>Properties</label>
      <p sx="text-sm text-muted-foreground">scale · opacity · translate-x · translate-y · rotate · <em>stagger</em> (reserved)</p>
    </column>
    <column gap="1">
      <label>Springs</label>
      <p sx="text-sm text-muted-foreground">spring:snappy · spring:gentle · spring:bouncy · spring:deliberate</p>
    </column>
    <column gap="1">
      <label>Tweens</label>
      <p sx="text-sm text-muted-foreground">fade · fade-out · press · release · pulse</p>
    </column>
  </grid>

  <alert tone="info">
    <strong>Reduced motion is not your problem.</strong> Every
    <code>motion:</code> respects <code>prefers-reduced-motion</code> — the
    JavaScript triggers check it, and the framework's global rule kills the CSS
    timelines. Honouring it is not something you can forget to do.
  </alert>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="default" href="/theming">← Theming</a>
    <a variant="solid" tone="primary" href="/behavior">Behavior →</a>
  </row>
</column>
