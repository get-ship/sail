<!--
  Ship UI landing — full-bleed dark marketing home (not a docs page).
  Giant rotating headline, crossing diagonal tickers, six-card bento to real
  sections, closing SHIP wordmark. Tokens + L1 + motion only.
-->

<controller>
// Hero word cycler (i → WORDS) and reactivity card counter (ticks).
import { interval, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

const WORDS = ['Batteries', 'Components', 'Styling', 'State', 'Animation', 'Themes']

export default controller((_ctx: ControllerScope<Scope>) => {
    const i = signal(0)
    interval(() => i.set((n) => (n + 1) % WORDS.length), 2600)

    const ticks = signal(0)
    interval(() => ticks.set((n) => n + 1), 1000)

    return {
      word: () => [WORDS[i()]],
      ticks,
    }
})
</controller>

<scope>
  // minW(0) + overflowX clip — diagonal tickers use negative inset; clip stops sideways growth.
  // Do NOT pair overflowX(hidden) with height-capped flex parent and leave Y visible:
  // CSS coerces Y→auto and sections become independent scrollports.
  style page = { box: w(full).maxW(full).minW(0).overflowX(hidden), paint: bg(surface-0) }
  style hero = { box: position(relative).padX(4).padT(12).padB(10).minW(0).overflowX(hidden), when: { tablet: { box: padX(8).padT(20).padB(16) } } }
  style glow = { box: left(0).right(0).top(0).h(64), paint: bgGlow(accent).z(base) }
  style heroInner = { box: position(relative).w(full).maxW(breakpoint-desktop).mx(auto).minW(0) }
  style eyebrow = { paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style display = { box: minW(0).maxW(full).overflow(hidden), paint: fg(foreground), text: fontSize(display).fontFamily(display).fontWeight(bold).tracking(tighter).leading(none).textAlign(center) }
  style line = { box: display(block).minW(0).overflow(hidden) }
  style lead = { box: maxW(breakpoint-mobile).minW(0), paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed).textAlign(center) }
  style ctaRow = { box: w(full).minW(0) }
  style cta = { box: padX(8), paint: radius(full) }
  style tickerSection = { box: position(relative).w(full).maxW(full).h(44).overflow(hidden).my(8).minW(0), paint: z(elevated) }
  style tickerBand = { box: position(absolute).left(-10).right(-10).top(24), paint: rotate(2) }
  style tickerDark = { box: padY(3).overflow(hidden), paint: borderTWidth(1).borderBWidth(1).borderT(border).borderB(border).bg(surface-2) }
  style tickerMono = { paint: fg(muted-foreground), text: fontFamily(mono).fontSize(sm).whitespace(nowrap) }
  style tickerBandTop = { box: position(absolute).left(-10).right(-10).top(12), paint: rotate(-2).z(popover) }
  style tickerPrimary = { box: padY(3).overflow(hidden), paint: bg(primary) }
  style tickerBold = { paint: fg(primary-foreground), text: fontWeight(bold).textCase(uppercase).tracking(wide).fontSize(md).whitespace(nowrap) }
  style bento = { box: padX(4).padT(6).padB(10).minW(0).overflowX(hidden), when: { tablet: { box: padX(8).padT(8).padB(12) } } }
  style grid = { box: w(full).maxW(breakpoint-desktop).mx(auto).minW(0) }
  style cardLink = { box: display(block).minW(0).w(full), paint: fg(foreground), text: underline(none), when: { hover: { paint: fg(foreground) } } }
  style cardFill = { box: h(full).overflow(hidden), when: { tablet: { box: minH(52) } } }
  style cardCol = { box: h(full).pad(4), when: { tablet: { box: pad(5) } } }
  style cardEyebrow = { paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style cardTitle = { box: mt(1), paint: fg(foreground), text: fontSize(md).fontWeight(semibold).leading(snug) }
  style cardBody = { box: mt(2), paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style cardFoot = { box: mt(auto).padT(5) }
  style metric = { paint: fg(foreground), text: fontFamily(mono).fontSize(2xl).fontWeight(semibold).nums(tabular).leading(none) }
  style metricUnit = { paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) }
  style swatch = { box: w(6).h(6), paint: radius(sm).borderWidth(1).border(border) }
  style statusLine = { paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) }
  style term = { box: padX(3).padY(2_5), paint: bg(surface-0).borderWidth(1).border(border).radius(md).fg(muted-foreground), text: fontFamily(mono).fontSize(xs) }
  style termAccent = { paint: fg(primary) }
  style foot = { box: padX(4).padT(8), when: { tablet: { box: padX(8).padT(12) } } }
  style footRow = { box: w(full).maxW(breakpoint-desktop).mx(auto).padB(6).gap(3), paint: borderBWidth(1).borderB(border).fg(muted-foreground), text: fontFamily(mono).fontSize(sm) }
  style footLink = { paint: fg(muted-foreground), text: underline(none), when: { hover: { paint: fg(primary) } } }
  style diamond = { box: display(inline-block).w(2).h(2), paint: rotate(45).bg(primary).radius(sm) }
  style wordmarkWrap = { box: w(full).maxW(full).minW(0).overflow(hidden) }
  style wordmark = { box: mb(-8).minW(0).maxW(full), paint: textGradient(primary).select(none), text: fontSize(display).fontFamily(display).fontWeight(bold).tracking(tighter).leading(none).textAlign(center) }
  style wordPrimary = { paint: fg(primary) }
  // Spring bar-loop — the MOTION bento's live proof that style.loop moves
  // translate-y (animation.spec.ts samples computed translate over time).
  style bar40 = { box: w(1_5).h(6), paint: radius(sm).bg(primary).opacity(40), animate: loop({ translate-y: 'spring:bouncy' }) }
  style bar60 = { box: w(1_5).h(10), paint: radius(sm).bg(primary).opacity(60), animate: loop({ translate-y: 'spring:gentle', stagger: '0.12s' }) }
  style bar70 = { box: w(1_5).h(8), paint: radius(sm).bg(primary).opacity(70), animate: loop({ translate-y: 'spring:bouncy', stagger: '0.12s' }) }
  style barFull = { box: w(1_5).h(14), paint: radius(sm).bg(primary), animate: loop({ translate-y: 'spring:gentle', stagger: '0.12s' }) }
  style bar60b = { box: w(1_5).h(9), paint: radius(sm).bg(primary).opacity(60), animate: loop({ translate-y: 'spring:bouncy', stagger: '0.12s' }) }
  style bar40b = { box: w(1_5).h(11), paint: radius(sm).bg(primary).opacity(40), animate: loop({ translate-y: 'spring:gentle', stagger: '0.12s' }) }
  // Empty prop bags for in-file partials (no params; expand at preprocess).
  type MotionBarsProps = {}
  type BentoCardProps = {}
  // Reactivity card needs the live signal — partials cannot read controller bindings.
  type BentoReactivityProps = {
    ticks: number
  }
</scope>

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
          <div style(swatch, { paint: bg(primary) })></div>
          <div style(swatch, { paint: bg(accent) })></div>
          <div style(swatch, { paint: bg(success) })></div>
          <div style(swatch, { paint: bg(warning) })></div>
          <div style(swatch, { paint: bg(info) })></div>
          <div style(swatch, { paint: bg(muted) })></div>
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

  <section style(hero, { box: shrink(0) })>
    <div style(glow)></div>

    <column gap="8" align="center" style(heroInner)>
      <p style(eyebrow)>SHIP UI · 0.2.0</p>

      <h1 style(display)>
        <span style(line)>
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

  <section style(tickerSection, { box: shrink(0) })>
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

  <section style(bento, { box: shrink(0) })>
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

  <section style(foot, { box: shrink(0) })>
    <row justify="between" align="center" style(footRow)>
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

  <div style(wordmarkWrap, { box: shrink(0) })>
    <p style(wordmark)>SHIP</p>
  </div>
</column>
