<!--
  Ship UI landing — full-bleed dark marketing home (not a docs page).
  Giant rotating headline, crossing diagonal tickers, six-card bento to real
  sections, closing SHIP wordmark. Tokens + L1 + motion only.
-->

<scope>
  // size({ min-w: 0 }) + overflowX clip — diagonal tickers use negative inset; clip stops sideways growth.
  // Do NOT pair overflow({ x: hidden }) with height-capped flex parent and leave Y visible:
  // CSS coerces Y→auto and sections become independent scrollports.
  style page = { box: size({ w: full, min-w: 0, max-w: full }).overflow({ x: hidden }), paint: fill({ color: surface-0 }) }
  style hero = { box: place({ position: relative }).space({ pad-x: 4, pad-t: 12, pad-b: 10 }).size({ min-w: 0 }).overflow({ x: hidden }), when: { tablet: { box: space({ pad-x: 8, pad-t: 20, pad-b: 16 }) } } }
  style glow = { box: place({ top: 0, left: 0, right: 0, z: base }).size({ h: 64 }), paint: fill({ glow: accent }) }
  style heroInner = { box: place({ position: relative }).size({ w: full, min-w: 0, max-w: breakpoint-desktop }).space({ mx: auto }) }
  style eyebrow = { paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style display = { box: size({ min-w: 0, max-w: full }).overflow({ all: hidden }), paint: ink({ color: foreground }), text: type({ family: display, size: display, weight: bold, leading: none, tracking: tighter, align: center }) }
  style line = { box: flow({ display: block }).size({ w: full, min-w: 0 }) }
  style wordLine = { box: flow({ display: block }).size({ w: full, min-w: 0 }).overflow({ all: hidden }), text: type({ wrap: truncate }) }
  style lead = { box: size({ min-w: 0, max-w: breakpoint-mobile }), paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed, align: center }) }
  style ctaRow = { box: size({ w: full, min-w: 0 }) }
  style cta = { box: space({ pad-x: 8 }), paint: radius(full) }
  style tickerSection = { box: place({ position: relative, z: elevated }).size({ w: full, h: 44, min-w: 0, max-w: full }).overflow({ all: hidden }).space({ my: 8 }) }
  style tickerBand = { box: place({ position: absolute, top: 24, left: -10, right: -10 }), paint: motion({ rotate: 2 }) }
  style tickerDark = { box: space({ pad-y: 3 }).overflow({ all: hidden }), paint: border({ color: border, width: 1, edge: y }).fill({ color: surface-2 }) }
  style tickerMono = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: sm, wrap: nowrap }) }
  style tickerBandTop = { box: place({ position: absolute, top: 12, left: -10, right: -10, z: popover }), paint: motion({ rotate: -2 }) }
  style tickerPrimary = { box: space({ pad-y: 3 }).overflow({ all: hidden }), paint: fill({ color: primary }) }
  style tickerBold = { paint: ink({ color: primary-foreground }), text: type({ size: md, weight: bold, tracking: wide, case: uppercase, wrap: nowrap }) }
  style bento = { box: space({ pad-x: 4, pad-t: 6, pad-b: 10 }).size({ min-w: 0 }).overflow({ x: hidden }), when: { tablet: { box: space({ pad-x: 8, pad-t: 8, pad-b: 12 }) } } }
  style grid = { box: size({ w: full, min-w: 0, max-w: breakpoint-desktop }).space({ mx: auto }) }
  style cardLink = { box: flow({ display: block }).size({ w: full, min-w: 0 }), paint: ink({ color: foreground }), text: type({ decoration: none }), when: { hover: { paint: ink({ color: foreground }) } } }
  style cardFill = { box: size({ h: full }).overflow({ all: hidden }), when: { tablet: { box: size({ min-h: 52 }) } } }
  style cardCol = { box: size({ h: full }).space({ pad: 4 }), when: { tablet: { box: space({ pad: 5 }) } } }
  style cardEyebrow = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style cardTitle = { box: space({ mt: 1 }), paint: ink({ color: foreground }), text: type({ size: md, weight: semibold, leading: snug }) }
  style cardBody = { box: space({ mt: 2 }), paint: ink({ color: muted-foreground }), text: type({ size: sm, leading: relaxed }) }
  style cardFoot = { box: space({ pad-t: 5, mt: auto }) }
  style metric = { paint: ink({ color: foreground }), text: type({family: mono, size: 2xl, weight: semibold, leading: none, nums: tabular}) }
  style metricUnit = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style swatch = { box: size(6), paint: radius(sm).border({ color: border, width: 1 }) }
  style statusLine = { box: size({ min-w: 0 }), paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs, wrap: truncate }) }
  style term = { box: space({ pad-x: 3, pad-y: 2_5 }), paint: fill({ color: surface-0 }).border({ color: border, width: 1 }).radius(md).ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style termAccent = { paint: ink({ color: primary }) }
  style foot = { box: space({ pad-x: 4, pad-t: 8 }), when: { tablet: { box: space({ pad-x: 8, pad-t: 12 }) } } }
  style footRow = { box: size({ w: full, max-w: breakpoint-desktop }).space({ pad-b: 6, mx: auto }), paint: border({ color: border, width: 1, edge: b }).ink({ color: muted-foreground }), text: type({ family: mono, size: sm }) }
  style footLink = { paint: ink({ color: muted-foreground }), text: type({ decoration: none }), when: { hover: { paint: ink({ color: primary }) } } }
  style diamond = { box: flow({ display: inline-block }).size(2), paint: motion({ rotate: 45 }).fill({ color: primary }).radius(sm) }
  style wordmarkWrap = { box: size({ w: full, min-w: 0, max-w: full }).overflow({ all: hidden }) }
  style wordmark = { box: space({ mb: -8 }).size({ min-w: 0, max-w: full }), paint: ink({ gradient: primary }).select(none), text: type({ family: display, size: display, weight: bold, leading: none, tracking: tighter, align: center }) }
  style wordPrimary = { paint: ink({ color: primary }) }
  // Spring bar-loop — the MOTION bento's live proof that style.loop moves
  // translate-y (animation.spec.ts samples computed translate over time).
  style bar40 = { box: size({ w: 1_5, h: 6 }), paint: radius(sm).fill({ color: primary }).filter({ opacity: 40 }), animate: loop({ translate-y: 'spring:bouncy' }) }
  style bar60 = { box: size({ w: 1_5, h: 10 }), paint: radius(sm).fill({ color: primary }).filter({ opacity: 60 }), animate: loop({ translate-y: 'spring:gentle', stagger: '0.12s' }) }
  style bar70 = { box: size({ w: 1_5, h: 8 }), paint: radius(sm).fill({ color: primary }).filter({ opacity: 70 }), animate: loop({ translate-y: 'spring:bouncy', stagger: '0.12s' }) }
  style barFull = { box: size({ w: 1_5, h: 14 }), paint: radius(sm).fill({ color: primary }), animate: loop({ translate-y: 'spring:gentle', stagger: '0.12s' }) }
  style bar60b = { box: size({ w: 1_5, h: 9 }), paint: radius(sm).fill({ color: primary }).filter({ opacity: 60 }), animate: loop({ translate-y: 'spring:bouncy', stagger: '0.12s' }) }
  style bar40b = { box: size({ w: 1_5, h: 11 }), paint: radius(sm).fill({ color: primary }).filter({ opacity: 40 }), animate: loop({ translate-y: 'spring:gentle', stagger: '0.12s' }) }
  // Empty prop bags for in-file partials (no params; expand at preprocess).
  type MotionBarsProps = {}
  type BentoCardProps = {}
  // Reactivity card needs the live signal — partials cannot read controller bindings.
  type BentoReactivityProps = {
    ticks: number
  }
</scope>

<controller>
// Hero word cycler (i → WORDS) and reactivity card counter (ticks).
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

const WORDS = ['Batteries', 'Components', 'Styling', 'State', 'Animation', 'Themes']

export default controller((_ctx: ControllerScope<Scope>) => {
    const i = signal(0)
    interval(() => i.set((n: number) => (n + 1) % WORDS.length), 2600)

    const ticks = signal(0)
    interval(() => ticks.set((n: number) => n + 1), 1000)

    return {
      word: () => [WORDS[i()]],
      ticks,
    }
})
</controller>

<!-- Motion foot reused by the animation bento card. -->
<partial motion-bars props: MotionBarsProps>
  <row gap="1" align="end" id="landing-motion-bars" style(cardFoot)>
    <div testid="landing-motion-bar" style(bar40)></div>
    <div style(bar60)></div>
    <div style(bar70)></div>
    <div style(barFull)></div>
    <div style(bar60b)></div>
    <div style(bar40b)></div>
  </row>
</partial>

<!-- Bento cards declared once; structural <when {tablet}> only swaps grid cols. -->
<partial bento-animation props: BentoCardProps>
  <a href="/docs/concepts/animation" style(cardLink)>
    <card interactive style(cardFill)>
      <column gap="0" style(cardCol)>
        <p style(cardEyebrow)>Animation</p>
        <strong style(cardTitle)>Motion as a language surface</strong>
        <p style(cardBody)>Springs, tweens, and stagger — declared in markup, not choreographed in JS.</p>
        <motion-bars></motion-bars>
      </column>
    </card>
  </a>
</partial>

<partial bento-theming props: BentoCardProps>
  <a href="/docs/concepts/theming" style(cardLink)>
    <card interactive style(cardFill)>
      <column gap="0" style(cardCol)>
        <p style(cardEyebrow)>Theming</p>
        <strong style(cardTitle)>Nine themes. One token graph.</strong>
        <p style(cardBody)>Swap theme and every surface, ink, and control re-resolves.</p>
        <row gap="2" style(cardFoot)>
          <div style(swatch, { paint: fill({ color: primary }) })></div>
          <div style(swatch, { paint: fill({ color: accent }) })></div>
          <div style(swatch, { paint: fill({ color: success }) })></div>
          <div style(swatch, { paint: fill({ color: warning }) })></div>
          <div style(swatch, { paint: fill({ color: info }) })></div>
          <div style(swatch, { paint: fill({ color: muted }) })></div>
        </row>
      </column>
    </card>
  </a>
</partial>

<partial bento-reactivity props: BentoReactivityProps>
  <a href="/docs/concepts/state" style(cardLink)>
    <card interactive style(cardFill)>
      <column gap="0" style(cardCol)>
        <p style(cardEyebrow)>Reactivity</p>
        <strong style(cardTitle)>Signals you can inspect</strong>
        <p style(cardBody)>Derived state and effects are first-class — no store glue layer.</p>
        <row gap="3" align="baseline" style(cardFoot)>
          <strong style(metric)>{ticks}</strong>
          <span style(metricUnit)>ticks · live signal</span>
        </row>
      </column>
    </card>
  </a>
</partial>

<partial bento-deck props: BentoCardProps>
  <a href="/docs/app" style(cardLink)>
    <card interactive style(cardFill)>
      <column gap="0" style(cardCol)>
        <p style(cardEyebrow)>Deck</p>
        <strong style(cardTitle)>SSR, routes, and HMR</strong>
        <p style(cardBody)>The app host is already wired — layouts, data, and hot reload included.</p>
        <column gap="2" style(cardFoot)>
          <row gap="2" align="center"><span dot tone="success"></span><span style(statusLine)>ssr · streaming</span></row>
          <row gap="2" align="center"><span dot tone="primary"></span><span style(statusLine)>file-based routes</span></row>
          <row gap="2" align="center"><span dot tone="info"></span><span style(statusLine)>hmr · sub-second</span></row>
        </column>
      </column>
    </card>
  </a>
</partial>

<partial bento-components props: BentoCardProps>
  <a href="/components" style(cardLink)>
    <card interactive style(cardFill)>
      <column gap="0" style(cardCol)>
        <p style(cardEyebrow)>Components</p>
        <strong style(cardTitle)>48 primitives, plus packs</strong>
        <p style(cardBody)>Forms, overlays, and feedback — themed and accessible by construction.</p>
        <row gap="2" align="center" style(cardFoot)>
          <badge tone="primary" size="sm">48 L1</badge>
          <badge tone="muted" size="sm" variant="soft">charts</badge>
          <badge tone="muted" size="sm" variant="soft">chat</badge>
          <badge tone="muted" size="sm" variant="soft">editor</badge>
        </row>
      </column>
    </card>
  </a>
</partial>

<partial bento-ai props: BentoCardProps>
  <a href="/docs/ai" style(cardLink)>
    <card interactive style(cardFill)>
      <column gap="0" style(cardCol)>
        <p style(cardEyebrow)>AI</p>
        <strong style(cardTitle)>Agents stay inside the language</strong>
        <p style(cardBody)>The compiler refuses invalid UI — so generated markup cannot drift the system.</p>
        <div style(term, cardFoot)>
          <p><span style(termAccent)>$</span> sail check app/</p>
          <p>ok · 0 refusals · 214 modules</p>
        </div>
      </column>
    </card>
  </a>
</partial>

<!-- cols not breakpoint-aware — structural when owns layout. -->
<column gap="0" style(page)>

  <section style(hero, { box: flow({ shrink: 0 }) })>
    <div style(glow)></div>

    <column gap="8" align="center" style(heroInner)>
      <p style(eyebrow)>SHIP UI · 0.2.0</p>

      <h1 style(display)>
        <span style(wordLine)>
          <for {word() as w} key={w}>
            <b style(wordPrimary)>{w}</b>
          </for>
        </span>
        <span style(line)>included.</span>
        <span style(line)>All of them.</span>
      </h1>

      <p style(lead)>
        One language with the whole interface inside it — built for agents to
        write and humans to review, with no way to drift off the design system.
      </p>

      <row gap="4" justify="center" align="center" style(ctaRow)>
        <a variant="solid" tone="primary" href="/docs/guide/getting-started" style(cta)>Get started</a>
        <a variant="outline" href="/docs/guide/why" style(cta)>Why Sail</a>
      </row>
    </column>
  </section>

  <section style(tickerSection, { box: flow({ shrink: 0 }) })>
    <div style(tickerBand)>
      <ticker duration="slow" reverse style(tickerDark)>
        <p style(tickerMono)>forms + validation&nbsp;&nbsp;·&nbsp;&nbsp;live data&nbsp;&nbsp;·&nbsp;&nbsp;springs + tweens&nbsp;&nbsp;·&nbsp;&nbsp;ssr&nbsp;&nbsp;·&nbsp;&nbsp;hot reload&nbsp;&nbsp;·&nbsp;&nbsp;design tokens&nbsp;&nbsp;·&nbsp;&nbsp;</p>
      </ticker>
    </div>
    <div style(tickerBandTop)>
      <ticker style(tickerPrimary)>
        <p style(tickerBold)>NO RAW CSS&nbsp;&nbsp;◆&nbsp;&nbsp;SIGNALS BUILT IN&nbsp;&nbsp;◆&nbsp;&nbsp;48 COMPONENTS&nbsp;&nbsp;◆&nbsp;&nbsp;9 THEMES&nbsp;&nbsp;◆&nbsp;&nbsp;A11Y BY CONSTRUCTION&nbsp;&nbsp;◆&nbsp;&nbsp;</p>
      </ticker>
    </div>
  </section>

  <section style(bento, { box: flow({ shrink: 0 }) })>
    <when {tablet}>
      <grid cols="3" gap="4" style(grid)>
        <bento-animation></bento-animation>
        <bento-theming></bento-theming>
        <bento-reactivity ticks={ticks()}></bento-reactivity>
        <bento-deck></bento-deck>
        <bento-components></bento-components>
        <bento-ai></bento-ai>
      </grid>
    {else => <grid cols="1" gap="3" style(grid)>
        <bento-animation></bento-animation>
        <bento-theming></bento-theming>
        <bento-reactivity ticks={ticks()}></bento-reactivity>
        <bento-deck></bento-deck>
        <bento-components></bento-components>
        <bento-ai></bento-ai>
      </grid>}</when>
  </section>

  <section style(foot, { box: flow({ shrink: 0 }) })>
    <row justify="between" align="center" gap="3" style(footRow)>
      <row gap="6" align="center">
        <a href="/docs/guide/introduction" style(footLink)>Docs</a>
        <a href="https://github.com/get-ship/sail" style(footLink)>GitHub</a>
      </row>
      <row gap="3" align="center">
        <span style(diamond)></span>
        <span>SHIP UI · 0.2.0</span>
      </row>
    </row>
  </section>

  <div style(wordmarkWrap, { box: flow({ shrink: 0 }) })>
    <p style(wordmark)>SHIP</p>
  </div>
</column>
