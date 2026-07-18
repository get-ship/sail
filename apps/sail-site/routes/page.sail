<!--
  Ship UI landing — design 1c KINETIC treatment carrying the REAL landing copy.
  A full-bleed dark marketing home (NOT a docs page):

    · a GIANT rotating headline — one teal word cycles ("Batteries · Components ·
      Styling · State · Motion · Themes") in front of the fixed "included. All of
      them." The cycle is the previous landing's proven WORDS + {#for}+motion:enter
      fade word-cycler, blown up to display size.
    · the signature TWO CROSSING DIAGONAL TICKER BANDS — a bright teal band tilted
      one way OVER a dark mono band tilted the other, overlapping so they visually
      cross, both running continuously (the stdlib <ticker> owns the loop).
    · a bento of SIX cards, each a real <a href> to an existing section, carrying
      that section's real headline and an on-brand micro-animation (spring bars,
      pulsing swatches, a live signal counter, deploy dots, live components, an
      agent terminal).
    · a closing giant SHIP wordmark in a teal→void gradient, cropped at the bottom.

  Everything is tokens + L1 primitives + motion — no <style>, no style=, no raw
  px/hex. The shell renders `/` full-bleed (fullscreen, no rail, no gutter) and
  defaults it to ship-dark.
-->

<controller>
// The page is CSS + motion; the two pieces of real state are the hero word
// cycler (i → which WORD is showing) and the reactivity card's live counter
// (ticks → a signal you can literally watch increment). Both are the previous
// landing's mechanisms, reused verbatim.
import { controller, signal, effect, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

const WORDS = ['Batteries', 'Components', 'Styling', 'State', 'Motion', 'Themes']

export default controller((_ctx: ControllerScope<Scope>) => {
    const i = signal(0)
    effect(() => {
      const t = setInterval(() => i.set((n) => (n + 1) % WORDS.length), 2600)
      return () => clearInterval(t)
    })

    // "State you can watch think." — a real signal ticking once a second.
    const ticks = signal(0)
    effect(() => {
      const t = setInterval(() => ticks.set((n) => n + 1), 1000)
      return () => clearInterval(t)
    })

    return {
      word: () => [WORDS[i()]],
      ticks,
    }
})
</controller>

<column gap="0" sx="w-full min-w-0 overflow-x-hidden bg-surface-0">

  <!-- ══ HERO — GIANT rotating type ═════════════════════════════════════ -->
  <section sx="relative px-8 pt-20 pb-16">
    <!-- soft radial glow behind the headline -->
    <div sx="absolute inset-x-0 top-0 h-64 bg-glow-accent pointer-events-none z-base"></div>

    <column gap="8" align="center" sx="relative w-full max-w-breakpoint-desktop mx-auto">
      <p sx="font-mono text-xs uppercase tracking-wide text-accent">SHIP UI · 0.2.0</p>

      <!-- GIANT three-line stack: the teal word cycles on its own line, then the
           fixed "included. / All of them." Each line is display-size and fits at
           1440 on its own, so the cycling word never overflows or reflows the rest. -->
      <h1 sx="text-display font-display font-bold tracking-tighter leading-none text-center text-foreground">
        <span sx="block">
          {#for w of word() key={w}}
            <b sx="text-primary" motion:enter={{ opacity: 'fade' }}>{w}</b>
          {/for}
        </span>
        <span sx="block">included.</span>
        <span sx="block">All of them.</span>
      </h1>

      <p sx="text-lg text-muted-foreground leading-relaxed text-center max-w-breakpoint-mobile">
        One language with the whole interface inside it — built for agents to
        write and humans to review, with no way to drift off the design system.
      </p>

      <row gap="4" justify="center" align="center" sx="w-full">
        <a variant="solid" tone="primary" href="/sail/getting-started" sx="rounded-full px-8" motion:press={{ scale: 'press' }}>Get started</a>
        <a variant="outline" href="/sail/why" sx="rounded-full px-8" motion:press={{ scale: 'press' }}>Why SAIL</a>
      </row>
    </column>
  </section>

  <!-- ══ TWO CROSSING DIAGONAL TICKER BANDS ════════════════════════════ -->
  <!-- Full-bleed: each band is wider than the viewport (-inset-x-10) and tilted
       opposite ways so they cross; the section clips the overhang. The <ticker>
       owns the seamless infinite scroll — both bands move, always. -->
  <section sx="relative w-full h-44 overflow-hidden my-8 z-elevated">
    <!-- dark mono band — tilted down-right, runs in reverse, sits lower -->
    <div sx="absolute -inset-x-10 top-24 rotate-2">
      <ticker duration="slow" reverse sx="border-t border-b border-border bg-surface-2 py-3">
        <p sx="font-mono text-sm text-muted-foreground whitespace-nowrap">forms + validation&nbsp;&nbsp;·&nbsp;&nbsp;live data&nbsp;&nbsp;·&nbsp;&nbsp;springs + tweens&nbsp;&nbsp;·&nbsp;&nbsp;ssr&nbsp;&nbsp;·&nbsp;&nbsp;hot reload&nbsp;&nbsp;·&nbsp;&nbsp;design tokens&nbsp;&nbsp;·&nbsp;&nbsp;</p>
      </ticker>
    </div>
    <!-- bright teal band — tilted up-right, on top where they cross -->
    <div sx="absolute -inset-x-10 top-12 -rotate-2 z-popover">
      <ticker sx="bg-primary py-3">
        <p sx="font-bold uppercase tracking-wide text-md text-primary-foreground whitespace-nowrap">NO RAW CSS&nbsp;&nbsp;◆&nbsp;&nbsp;SIGNALS BUILT IN&nbsp;&nbsp;◆&nbsp;&nbsp;48 COMPONENTS&nbsp;&nbsp;◆&nbsp;&nbsp;9 THEMES&nbsp;&nbsp;◆&nbsp;&nbsp;A11Y BY CONSTRUCTION&nbsp;&nbsp;◆&nbsp;&nbsp;</p>
      </ticker>
    </div>
  </section>

  <!-- ══ BENTO — six real links, each with a micro-animation ═══════════ -->
  <section sx="px-8 pt-8 pb-12">
    <grid cols="3" gap="4" sx="w-full max-w-breakpoint-desktop mx-auto">

      <!-- MOTION → /sail/motion : a spring bar-loop -->
      <a href="/sail/motion" sx="block no-underline min-w-0">
        <card interactive sx="h-full overflow-hidden">
          <column gap="4" sx="h-full">
            <column gap="1">
              <p sx="font-mono text-xs uppercase tracking-wide text-accent">Motion</p>
              <strong sx="text-lg text-foreground leading-snug">Real physics. Zero choreography code.</strong>
            </column>
            <row gap="2" align="end" sx="h-16 mt-auto">
              <div sx="w-4 h-8 rounded-sm bg-primary" motion:loop={{ translate-y: 'spring:bouncy' }}></div>
              <div sx="w-4 h-14 rounded-sm bg-primary opacity-60" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}></div>
              <div sx="w-4 h-11 rounded-sm bg-primary" motion:loop={{ translate-y: 'spring:bouncy', stagger: '0.12s' }}></div>
              <div sx="w-4 h-16 rounded-sm bg-primary opacity-60" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}></div>
              <div sx="w-4 h-10 rounded-sm bg-primary" motion:loop={{ translate-y: 'spring:bouncy', stagger: '0.12s' }}></div>
            </row>
          </column>
        </card>
      </a>

      <!-- THEMING → /sail/theming : pulsing repaint swatches -->
      <a href="/sail/theming" sx="block no-underline min-w-0">
        <card interactive sx="h-full overflow-hidden">
          <column gap="4" sx="h-full">
            <column gap="1">
              <p sx="font-mono text-xs uppercase tracking-wide text-accent">Theming</p>
              <strong sx="text-lg text-foreground leading-snug">Press a button. The whole site repaints.</strong>
            </column>
            <row gap="2" sx="mt-auto">
              <div sx="w-10 h-10 rounded-md bg-primary" motion:loop={{ opacity: 'pulse' }}></div>
              <div sx="w-10 h-10 rounded-md bg-success" motion:loop={{ opacity: 'pulse', stagger: '0.15s' }}></div>
              <div sx="w-10 h-10 rounded-md bg-warning" motion:loop={{ opacity: 'pulse', stagger: '0.15s' }}></div>
              <div sx="w-10 h-10 rounded-md bg-info" motion:loop={{ opacity: 'pulse', stagger: '0.15s' }}></div>
            </row>
          </column>
        </card>
      </a>

      <!-- REACTIVITY → /sail/language/state : a live signal counter -->
      <a href="/sail/language/state" sx="block no-underline min-w-0">
        <card interactive sx="h-full overflow-hidden">
          <column gap="4" sx="h-full">
            <column gap="1">
              <p sx="font-mono text-xs uppercase tracking-wide text-accent">Reactivity</p>
              <strong sx="text-lg text-foreground leading-snug">State you can watch think.</strong>
            </column>
            <row gap="3" align="baseline" sx="mt-auto">
              <strong sx="font-mono text-display text-primary tabular-nums leading-none">{ticks()}</strong>
              <span sx="font-mono text-xs text-muted-foreground">signal · live</span>
            </row>
          </column>
        </card>
      </a>

      <!-- DECK → /deck : pulsing build/deploy status dots -->
      <a href="/deck" sx="block no-underline min-w-0">
        <card interactive sx="h-full overflow-hidden">
          <column gap="4" sx="h-full">
            <column gap="1">
              <p sx="font-mono text-xs uppercase tracking-wide text-accent">Deck</p>
              <strong sx="text-lg text-foreground leading-snug">The app layer is already wired.</strong>
            </column>
            <column gap="2" sx="mt-auto font-mono text-xs text-muted-foreground">
              <row gap="2" align="center"><span dot tone="primary" size="sm" pulse></span><span>build 214</span></row>
              <row gap="2" align="center"><span dot tone="success" size="sm" pulse></span><span>deployed</span></row>
              <row gap="2" align="center"><span dot tone="info" size="sm" pulse></span><span>live · ssr</span></row>
            </column>
          </column>
        </card>
      </a>

      <!-- COMPONENTS → /components : live components peek -->
      <a href="/components" sx="block no-underline min-w-0">
        <card interactive sx="h-full overflow-hidden relative">
          <column gap="4" sx="h-full">
            <column gap="1">
              <p sx="font-mono text-xs uppercase tracking-wide text-accent">Components</p>
              <strong sx="text-lg text-foreground leading-snug">The hard parts are already correct.</strong>
            </column>
            <row gap="3" align="center" sx="mt-auto">
              <switch checked tone="primary"></switch>
              <spinner></spinner>
              <badge tone="success">live</badge>
              <chip>chip</chip>
            </row>
          </column>
          <span sx="absolute bottom-4 right-4 font-mono text-xs text-muted-foreground">48 + packs</span>
        </card>
      </a>

      <!-- AI → /ai : an agent-compile terminal with a blinking cursor -->
      <a href="/ai" sx="block no-underline min-w-0">
        <card interactive sx="h-full overflow-hidden bg-tone-primary-bg border border-primary">
          <column gap="4" sx="h-full">
            <column gap="1">
              <p sx="font-mono text-xs uppercase tracking-wide text-accent">AI</p>
              <strong sx="text-lg text-foreground leading-snug">Agents can't break what the language won't allow.</strong>
            </column>
            <column gap="1" sx="mt-auto font-mono text-sm bg-surface-0 border border-border rounded-lg px-4 py-3">
              <row gap="2" align="center"><span sx="text-muted-foreground">$</span><span sx="text-foreground">agent → sail compile</span></row>
              <row gap="1" align="center">
                <span sx="text-primary">✓ 0 errors · themed · a11y</span>
                <span sx="inline-block w-1 h-4 bg-primary" motion:loop={{ opacity: 'pulse' }}></span>
              </row>
            </column>
          </column>
        </card>
      </a>

    </grid>
  </section>

  <!-- ══ FOOTER + closing giant SHIP wordmark ══════════════════════════ -->
  <section sx="px-8 pt-12">
    <row justify="between" align="center" sx="w-full max-w-breakpoint-desktop mx-auto pb-6 border-b border-border font-mono text-sm text-muted-foreground">
      <row gap="6" align="center">
        <a href="/sail/overview" sx="text-muted-foreground hover:text-primary">Docs</a>
        <a href="https://github.com" sx="text-muted-foreground hover:text-primary">GitHub</a>
        <a href="https://discord.com" sx="text-muted-foreground hover:text-primary">Discord</a>
      </row>
      <row gap="3" align="center">
        <!-- diamond logo mark: a tilted teal square with the brand glow -->
        <span sx="inline-block w-3 h-3 rotate-45 bg-primary rounded-sm"></span>
        <span>SHIP UI · 0.2.0</span>
      </row>
    </row>
  </section>

  <!-- teal→void gradient wordmark, full-bleed and cropped at the bottom edge -->
  <div sx="w-full overflow-hidden">
    <p sx="text-display font-display font-bold tracking-tighter leading-none text-center text-gradient-primary select-none -mb-8" motion:scroll={{ scale: 'spring:deliberate' }}>SHIP</p>
  </div>
</column>
