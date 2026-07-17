<!--
  SAIL landing — design 1c, the kinetic "Ship UI home". A full-bleed dark
  marketing hero (NOT a docs page): a two-line display headline behind a soft
  radial glow, two CTAs, then the signature TWO CROSSING DIAGONAL TICKER BANDS
  (a bright teal band angled one way over a darker mono band angled the other,
  both running continuously and full-width). Below, an asymmetric bento of live
  micro-demos — an animating bar chart, pulsing deploy dots, a real toggle, a
  blinking-cursor terminal — and a closing giant SHIP wordmark in a teal→void
  gradient, cropped at the very bottom.

  Everything here is tokens + L1 primitives + motion. The tickers are the
  stdlib <ticker> (its own seamless loop); the rotation, glow, gradient
  wordmark, bars and cursor are ordinary sx atoms (rotate-*, bg-glow-accent,
  text-gradient-primary) and motion:loop — no <style>, no style=, no raw px/hex.
  The shell renders `/` full-bleed (fullscreen, no rail, no reading gutter).
-->

<controller>
// The page is almost entirely CSS/motion — the one piece of real state is the
// bento's "it works" switch, a genuine interactive toggle the reader can flip.
import { controller, signal, type ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

export default controller((_ctx: ViewScope<Scope>) => {
    const on = signal(true)
    const toggle = (): void => on.set((v) => !v)
    return { on, toggle }
})
</controller>

<column gap="0" sx="w-full min-w-0 overflow-x-hidden bg-surface-0">

  <!-- ══ HERO ═══════════════════════════════════════════════════════════ -->
  <section sx="relative px-8 pt-20 pb-16">
    <!-- soft radial glow behind the headline -->
    <div sx="absolute inset-x-0 top-0 h-64 bg-glow-accent pointer-events-none z-base"></div>

    <column gap="8" align="center" sx="relative w-full max-w-breakpoint-desktop mx-auto">
      <!-- two-line kinetic headline: "Agents" then "You review." offset right -->
      <column gap="0" align="center" sx="w-full">
        <p sx="w-full text-display font-display font-bold tracking-tighter leading-none text-foreground text-left pl-6">Agents</p>
        <p sx="w-full text-display font-display font-bold tracking-tighter leading-none text-foreground text-right pr-6">You review.</p>
      </column>

      <p sx="text-lg text-muted-foreground leading-relaxed text-center max-w-breakpoint-mobile">
        One confined language for the whole interface — so agent-written UI
        compiles into guarantees, not surprises.
      </p>

      <row gap="4" justify="center" align="center" sx="w-full">
        <a variant="solid" tone="primary" href="/getting-started" sx="rounded-full px-8" motion:press={{ scale: 'press' }}>Get started</a>
        <a variant="outline" href="/getting-started" sx="rounded-full px-8 font-mono" motion:press={{ scale: 'press' }}>npx create-ship</a>
      </row>
    </column>
  </section>

  <!-- ══ TWO CROSSING DIAGONAL TICKER BANDS ════════════════════════════ -->
  <!-- Full-bleed: each band is wider than the viewport (-inset-x-10) and
       tilted opposite ways so they cross; the section clips the overhang. The
       <ticker> owns the seamless infinite scroll (both bands move, always). -->
  <section sx="relative w-full h-44 overflow-hidden my-8 z-elevated">
    <!-- darker mono band — angled down-right, runs in reverse, sits lower -->
    <div sx="absolute -inset-x-10 top-24 rotate-2">
      <ticker duration="slow" reverse sx="border-t border-b border-border bg-surface-2 py-3">
        <p sx="font-mono text-sm text-muted-foreground whitespace-nowrap">hot reload&nbsp;&nbsp;·&nbsp;&nbsp;forms + validation&nbsp;&nbsp;·&nbsp;&nbsp;swr data&nbsp;&nbsp;·&nbsp;&nbsp;server functions&nbsp;&nbsp;·&nbsp;&nbsp;ssr&nbsp;&nbsp;·&nbsp;&nbsp;animations&nbsp;&nbsp;·&nbsp;&nbsp;</p>
      </ticker>
    </div>
    <!-- bright teal band — angled up-right, on top where they cross -->
    <div sx="absolute -inset-x-10 top-10 -rotate-2 z-popover">
      <ticker sx="bg-primary py-3">
        <p sx="font-bold uppercase tracking-wide text-md text-primary-foreground whitespace-nowrap">9 themes&nbsp;&nbsp;◆&nbsp;&nbsp;a11y by construction&nbsp;&nbsp;◆&nbsp;&nbsp;no raw css&nbsp;&nbsp;◆&nbsp;&nbsp;signals built in&nbsp;&nbsp;◆&nbsp;&nbsp;tokens&nbsp;&nbsp;◆&nbsp;&nbsp;</p>
      </ticker>
    </div>
  </section>

  <!-- ══ BENTO — asymmetric grid of live micro-demos ═══════════════════ -->
  <section sx="px-8 pt-8 pb-12">
    <row gap="4" align="start" sx="w-full max-w-breakpoint-desktop mx-auto">

      <!-- LEFT column: the tall SAIL card + the AI terminal beneath it -->
      <column gap="4" sx="basis-half grow-1 min-w-0">
        <!-- SAIL reactive language — animated bar chart + signals link -->
        <card sx="grow-1 min-w-0 min-h-64 overflow-hidden">
          <column gap="6" sx="h-full">
            <column gap="2">
              <p><strong sx="text-2xl text-primary">SAIL</strong> <span sx="font-mono text-xs text-muted-foreground">reactive language</span></p>
              <p sx="text-sm text-muted-foreground leading-relaxed max-w-64">Markup that already knows how to style, animate and react. The framework has the rest.</p>
            </column>
            <row gap="2" align="end" sx="h-20 mt-auto">
              <div sx="w-5 h-9 rounded-sm bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}></div>
              <div sx="w-5 h-14 rounded-sm bg-primary opacity-60" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}></div>
              <div sx="w-5 h-11 rounded-sm bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}></div>
              <div sx="w-5 h-16 rounded-sm bg-primary opacity-60" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}></div>
              <div sx="w-5 h-8 rounded-sm bg-primary" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}></div>
              <div sx="w-5 h-12 rounded-sm bg-primary opacity-60" motion:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}></div>
              <span sx="ml-auto self-end font-mono text-xs text-muted-foreground">signals →</span>
            </row>
          </column>
        </card>

        <!-- AI agent tooling — accent-tinted card with a live terminal -->
        <card sx="grow-1 min-w-0 bg-tone-primary-bg border border-primary">
          <row gap="4" justify="between" align="center">
            <column gap="2" sx="min-w-0 basis-56 grow-1">
              <p><strong sx="text-xl text-primary">AI</strong> <span sx="font-mono text-xs text-muted-foreground">agent tooling</span></p>
              <p sx="text-sm text-muted-foreground leading-relaxed">MCP server, scored authoring eval, and a compiler that rejects what it can't guarantee.</p>
            </column>
            <column gap="1" sx="shrink-0 font-mono text-sm bg-surface-0 border border-border rounded-lg px-4 py-3">
              <row gap="2" align="center"><span sx="text-muted-foreground">$</span><span sx="text-foreground">agent → ship compile</span></row>
              <row gap="1" align="center">
                <span sx="text-primary">✓ 0 errors · themed · a11y</span>
                <span sx="inline-block w-1 h-4 bg-primary" motion:loop={{ opacity: 'pulse' }}></span>
              </row>
            </column>
          </row>
        </card>
      </column>

      <!-- RIGHT column: Deck card, then Components + toggle side by side -->
      <column gap="4" sx="basis-half grow-1 min-w-0">
        <!-- Deck app framework — pulsing status readout -->
        <card sx="min-w-0">
          <row gap="4" justify="between" align="center">
            <column gap="2" sx="min-w-0 basis-56 grow-1">
              <p><strong sx="text-xl text-primary">Deck</strong> <span sx="font-mono text-xs text-muted-foreground">app framework</span></p>
              <p sx="text-sm text-muted-foreground leading-relaxed">SSR, routing, bundling, server functions, live data.</p>
            </column>
            <column gap="1" sx="shrink-0 font-mono text-xs text-primary">
              <row gap="2" align="center"><span dot tone="primary" size="sm" pulse></span><span>build 214</span></row>
              <row gap="2" align="center"><span dot tone="primary" size="sm" pulse></span><span>deployed</span></row>
              <row gap="2" align="center"><span dot tone="primary" size="sm" pulse></span><span>p99 41ms</span></row>
            </column>
          </row>
        </card>

        <row gap="4" align="stretch">
          <!-- Components — catalog readout -->
          <card sx="grow-1 basis-0 min-w-0 relative overflow-hidden">
            <column gap="3">
              <strong sx="text-lg text-primary">Components</strong>
              <column gap="1" sx="font-mono text-sm text-muted-foreground">
                <span>toast · queued</span>
                <span>tabs · animated</span>
              </column>
            </column>
            <span sx="absolute bottom-4 right-4 font-mono text-xs text-muted-foreground">48 + packs</span>
          </card>

          <!-- the small interactive card: a real toggle -->
          <card sx="shrink-0 basis-28">
            <center sx="h-full">
              <column gap="3" align="center">
                <switch bind:checked={on} tone="primary"></switch>
                <span sx="font-mono text-xs text-muted-foreground">it works</span>
              </column>
            </center>
          </card>
        </row>
      </column>
    </row>
  </section>

  <!-- ══ FOOTER + giant SHIP wordmark ══════════════════════════════════ -->
  <section sx="px-8 pt-12">
    <row justify="between" align="center" sx="w-full max-w-breakpoint-desktop mx-auto pb-6 border-b border-border font-mono text-sm text-muted-foreground">
      <row gap="6" align="center">
        <a href="/overview" sx="text-muted-foreground hover:text-primary">Docs</a>
        <a href="https://github.com" sx="text-muted-foreground hover:text-primary">GitHub</a>
        <a href="https://discord.com" sx="text-muted-foreground hover:text-primary">Discord</a>
      </row>
      <span>v0.2.0</span>
    </row>
  </section>

  <!-- teal→void gradient wordmark, full-bleed and cropped at the bottom edge -->
  <div sx="w-full overflow-hidden">
    <p sx="text-display font-display font-bold tracking-tighter leading-none text-center text-gradient-primary select-none -mb-8">SHIP</p>
  </div>
</column>
