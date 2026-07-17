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

import { controller, signal, type ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { code } from './snippets'

export default controller((_ctx: ViewScope<Scope>) => {
  const open = signal(true)
  const dealt = signal(true)
  const replay = (): void => {
    dealt.set(false)
    setTimeout(() => dealt.set(true), 450)
  }
  return {
    code,
    open,
    dealt,
    replay,
    hand: signal(['A', 'K', 'Q', 'J', '10']),
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
          <badge tone="primary" motion:hover={{ opacity: 'pulse' }}>hover · pulse</badge>
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
