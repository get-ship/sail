<!--
  Ship UI landing — design 1c KINETIC treatment carrying the REAL landing copy.
  A full-bleed dark marketing home (NOT a docs page):

    · a GIANT rotating headline — one teal word cycles ("Batteries · Components ·
      Styling · State · Motion · Themes") in front of the fixed "included. All of
      them." The cycle is the previous landing's proven WORDS + {#for}+style.enter
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
  defaults it to ship (dark mode).
-->

<controller>
// The page is CSS + motion; the two pieces of real state are the hero word
// cycler (i → which WORD is showing) and the reactivity card's live counter
// (ticks → a signal you can literally watch increment). Both are the previous
// landing's mechanisms, reused verbatim.
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

const WORDS = ['Batteries', 'Components', 'Styling', 'State', 'Animation', 'Themes']

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

    // grid cols is not responsive in the language — swap 1 vs 3 tracks via matchMedia.
    const isWide = signal(
      typeof window !== 'undefined' && window.matchMedia('(min-width: 768px)').matches,
    )
    mount(() => {
      if (typeof window === 'undefined') return
      const mq = window.matchMedia('(min-width: 768px)')
      const onChange = (): void => isWide.set(mq.matches)
      onChange()
      mq.addEventListener('change', onChange)
      return () => mq.removeEventListener('change', onChange)
    })

    return {
      word: () => [WORDS[i()]],
      ticks,
      isWide,
    }
})
</controller>

<scope>
  // minW(0) + overflowX clip on the page root — diagonal tickers use
  // left/right negative inset; without a hard clip the document grows sideways.
  const page = style.w(full).maxW(full).minW(0).overflowX(hidden).overflowY(visible).bg(surface-0)
  const hero = style.position(relative).padX(4).padT(12).padB(10).minW(0).overflowX(hidden).when({
    tablet: style.padX(8).padT(20).padB(16),
  })
  const glow = style.bgGlow(accent).left(0).right(0).top(0).h(64).z(base)
  const heroInner = style.position(relative).w(full).maxW(breakpoint-desktop).mx(auto).minW(0)
  const eyebrow = style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent)
  // minW(0) keeps the display word from forcing the page wider than the viewport.
  const display = style.fontSize(display).fontFamily(display).fontWeight(bold).tracking(tighter).leading(none).textAlign(center).fg(foreground).minW(0).maxW(full).overflow(hidden)
  const line = style.display(block).minW(0).overflow(hidden)
  const lead = style.fontSize(lg).fg(muted-foreground).leading(relaxed).textAlign(center).maxW(breakpoint-mobile).minW(0)
  const ctaRow = style.w(full).minW(0)
  const cta = style.radius(full).padX(8)
  const tickerSection = style.position(relative).w(full).maxW(full).h(44).overflow(hidden).my(8).z(elevated).minW(0)
  const tickerBand = style.position(absolute).left(-10).right(-10).top(24).rotate(2)
  const tickerDark = style.borderTWidth(1).borderBWidth(1).borderT(border).borderB(border).bg(surface-2).padY(3).overflow(hidden)
  const tickerMono = style.fontFamily(mono).fontSize(sm).fg(muted-foreground).whitespace(nowrap)
  const tickerBandTop = style.position(absolute).left(-10).right(-10).top(12).rotate(-2).z(popover)
  const tickerPrimary = style.bg(primary).padY(3).overflow(hidden)
  const tickerBold = style.fontWeight(bold).textCase(uppercase).tracking(wide).fontSize(md).fg(primary-foreground).whitespace(nowrap)
  const bento = style.padX(4).padT(6).padB(10).minW(0).overflowX(hidden).when({
    tablet: style.padX(8).padT(8).padB(12),
  })
  const grid = style.w(full).maxW(breakpoint-desktop).mx(auto).minW(0)
  const cardLink = style.display(block).underline(none).minW(0).w(full).fg(foreground).when({ hover: style.fg(foreground) })
  // Equal-height bento cells — quiet surface, real pad. minH only from tablet up
  // so phone cards don't become full-viewport towers.
  const cardFill = style.h(full).overflow(hidden).when({ tablet: style.minH(52) })
  const cardCol = style.h(full).pad(4).when({ tablet: style.pad(5) })
  const cardEyebrow = style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(muted-foreground)
  const cardTitle = style.fontSize(md).fontWeight(semibold).fg(foreground).leading(snug).mt(1)
  const cardBody = style.fontSize(sm).fg(muted-foreground).leading(relaxed).mt(2)
  const cardFoot = style.mt(auto).padT(5)
  const metric = style.fontFamily(mono).fontSize(2xl).fontWeight(semibold).fg(foreground).nums(tabular).leading(none)
  const metricUnit = style.fontFamily(mono).fontSize(xs).fg(muted-foreground)
  const swatch = style.w(6).h(6).radius(sm).borderWidth(1).border(border)
  const statusLine = style.fontFamily(mono).fontSize(xs).fg(muted-foreground)
  const term = style.fontFamily(mono).fontSize(xs).bg(surface-0).borderWidth(1).border(border).radius(md).padX(3).padY(2_5).fg(muted-foreground)
  const termAccent = style.fg(primary)
  const foot = style.padX(4).padT(8).when({ tablet: style.padX(8).padT(12) })
  const footRow = style.w(full).maxW(breakpoint-desktop).mx(auto).padB(6).borderBWidth(1).borderB(border).fontFamily(mono).fontSize(sm).fg(muted-foreground).gap(3)
  const footLink = style.fg(muted-foreground).underline(none).when({ hover: style.fg(primary) })
  const diamond = style.display(inline-block).w(2).h(2).rotate(45).bg(primary).radius(sm)
  const wordmarkWrap = style.w(full).maxW(full).minW(0).overflow(hidden)
  const wordmark = style.fontSize(display).fontFamily(display).fontWeight(bold).tracking(tighter).leading(none).textAlign(center).textGradient(primary).select(none).mb(-8).minW(0).maxW(full)
</scope>

<column gap="0" sx={page}>

  <section sx={hero}>
    <div sx={glow}></div>

    <column gap="8" align="center" sx={heroInner}>
      <p sx={eyebrow}>SHIP UI · 0.2.0</p>

      <h1 sx={display}>
        <span sx={line}>
          <for {word()} as="w" key={w}>
            <b sx={style.fg(primary)}>{w}</b>
          </for>
        </span>
        <span sx={line}>included.</span>
        <span sx={line}>All of them.</span>
      </h1>

      <p sx={lead}>
        One language with the whole interface inside it — built for agents to
        write and humans to review, with no way to drift off the design system.
      </p>

      <row gap="4" justify="center" align="center" sx={ctaRow}>
        <a variant="solid" tone="primary" href="/sail/getting-started" sx={cta}>Get started</a>
        <a variant="outline" href="/sail/why" sx={cta}>Why Sail</a>
      </row>
    </column>
  </section>

  <section sx={tickerSection}>
    <div sx={tickerBand}>
      <ticker duration="slow" reverse sx={tickerDark}>
        <p sx={tickerMono}>forms + validation&nbsp;&nbsp;·&nbsp;&nbsp;live data&nbsp;&nbsp;·&nbsp;&nbsp;springs + tweens&nbsp;&nbsp;·&nbsp;&nbsp;ssr&nbsp;&nbsp;·&nbsp;&nbsp;hot reload&nbsp;&nbsp;·&nbsp;&nbsp;design tokens&nbsp;&nbsp;·&nbsp;&nbsp;</p>
      </ticker>
    </div>
    <div sx={tickerBandTop}>
      <ticker sx={tickerPrimary}>
        <p sx={tickerBold}>NO RAW CSS&nbsp;&nbsp;◆&nbsp;&nbsp;SIGNALS BUILT IN&nbsp;&nbsp;◆&nbsp;&nbsp;48 COMPONENTS&nbsp;&nbsp;◆&nbsp;&nbsp;9 THEMES&nbsp;&nbsp;◆&nbsp;&nbsp;A11Y BY CONSTRUCTION&nbsp;&nbsp;◆&nbsp;&nbsp;</p>
      </ticker>
    </div>
  </section>

  <section sx={bento}>
    <!--
      <grid cols> is not breakpoint-aware. Duplicate the six cards only to
      swap track count — language gap, not content gap.
    -->
    <if {isWide()}>
      <grid cols="3" gap="4" sx={grid}>
        <a href="/sail/language/animation" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>Animation</p>
              <strong sx={cardTitle}>Motion as a language surface</strong>
              <p sx={cardBody}>Springs, tweens, and stagger — declared in markup, not choreographed in JS.</p>
              <row gap="1" align="end" sx={cardFoot} data-testid="landing-motion-bars">
                <div data-testid="landing-motion-bar" sx={style.w(1_5).h(6).radius(sm).bg(primary).opacity(40)}></div>
                <div sx={style.w(1_5).h(10).radius(sm).bg(primary).opacity(60)}></div>
                <div sx={style.w(1_5).h(8).radius(sm).bg(primary).opacity(70)}></div>
                <div sx={style.w(1_5).h(14).radius(sm).bg(primary)}></div>
                <div sx={style.w(1_5).h(9).radius(sm).bg(primary).opacity(60)}></div>
                <div sx={style.w(1_5).h(11).radius(sm).bg(primary).opacity(40)}></div>
              </row>
            </column>
          </card>
        </a>
        <a href="/sail/language/theming" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>Theming</p>
              <strong sx={cardTitle}>Nine themes. One token graph.</strong>
              <p sx={cardBody}>Swap data-theme and every surface, ink, and control re-resolves.</p>
              <row gap="2" sx={cardFoot}>
                <div sx={[swatch, style.bg(primary)]}></div>
                <div sx={[swatch, style.bg(accent)]}></div>
                <div sx={[swatch, style.bg(success)]}></div>
                <div sx={[swatch, style.bg(warning)]}></div>
                <div sx={[swatch, style.bg(info)]}></div>
                <div sx={[swatch, style.bg(muted)]}></div>
              </row>
            </column>
          </card>
        </a>
        <a href="/sail/language/state" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>Reactivity</p>
              <strong sx={cardTitle}>Signals you can inspect</strong>
              <p sx={cardBody}>Derived state and effects are first-class — no store glue layer.</p>
              <row gap="3" align="baseline" sx={cardFoot}>
                <strong sx={metric}>{ticks()}</strong>
                <span sx={metricUnit}>ticks · live signal</span>
              </row>
            </column>
          </card>
        </a>
        <a href="/deck" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>Deck</p>
              <strong sx={cardTitle}>SSR, routes, and HMR</strong>
              <p sx={cardBody}>The app host is already wired — layouts, data, and hot reload included.</p>
              <column gap="2" sx={cardFoot}>
                <row gap="2" align="center"><span dot tone="success"></span><span sx={statusLine}>ssr · streaming</span></row>
                <row gap="2" align="center"><span dot tone="primary"></span><span sx={statusLine}>file-based routes</span></row>
                <row gap="2" align="center"><span dot tone="info"></span><span sx={statusLine}>hmr · sub-second</span></row>
              </column>
            </column>
          </card>
        </a>
        <a href="/components" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>Components</p>
              <strong sx={cardTitle}>48 primitives, plus packs</strong>
              <p sx={cardBody}>Forms, overlays, and feedback — themed and accessible by construction.</p>
              <row gap="2" align="center" sx={cardFoot}>
                <badge tone="primary" size="sm">48 L1</badge>
                <badge tone="muted" size="sm" variant="soft">charts</badge>
                <badge tone="muted" size="sm" variant="soft">chat</badge>
                <badge tone="muted" size="sm" variant="soft">editor</badge>
              </row>
            </column>
          </card>
        </a>
        <a href="/ai" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>AI</p>
              <strong sx={cardTitle}>Agents stay inside the language</strong>
              <p sx={cardBody}>The compiler refuses invalid UI — so generated markup cannot drift the system.</p>
              <div sx={[term, cardFoot]}>
                <p><span sx={termAccent}>$</span> sail check app/</p>
                <p>ok · 0 refusals · 214 modules</p>
              </div>
            </column>
          </card>
        </a>
      </grid>
    <else>
      <grid cols="1" gap="3" sx={grid}>
        <a href="/sail/language/animation" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>Animation</p>
              <strong sx={cardTitle}>Motion as a language surface</strong>
              <p sx={cardBody}>Springs, tweens, and stagger — declared in markup, not choreographed in JS.</p>
              <row gap="1" align="end" sx={cardFoot} data-testid="landing-motion-bars">
                <div data-testid="landing-motion-bar" sx={style.w(1_5).h(6).radius(sm).bg(primary).opacity(40)}></div>
                <div sx={style.w(1_5).h(10).radius(sm).bg(primary).opacity(60)}></div>
                <div sx={style.w(1_5).h(8).radius(sm).bg(primary).opacity(70)}></div>
                <div sx={style.w(1_5).h(14).radius(sm).bg(primary)}></div>
                <div sx={style.w(1_5).h(9).radius(sm).bg(primary).opacity(60)}></div>
                <div sx={style.w(1_5).h(11).radius(sm).bg(primary).opacity(40)}></div>
              </row>
            </column>
          </card>
        </a>
        <a href="/sail/language/theming" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>Theming</p>
              <strong sx={cardTitle}>Nine themes. One token graph.</strong>
              <p sx={cardBody}>Swap data-theme and every surface, ink, and control re-resolves.</p>
              <row gap="2" sx={cardFoot}>
                <div sx={[swatch, style.bg(primary)]}></div>
                <div sx={[swatch, style.bg(accent)]}></div>
                <div sx={[swatch, style.bg(success)]}></div>
                <div sx={[swatch, style.bg(warning)]}></div>
                <div sx={[swatch, style.bg(info)]}></div>
                <div sx={[swatch, style.bg(muted)]}></div>
              </row>
            </column>
          </card>
        </a>
        <a href="/sail/language/state" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>Reactivity</p>
              <strong sx={cardTitle}>Signals you can inspect</strong>
              <p sx={cardBody}>Derived state and effects are first-class — no store glue layer.</p>
              <row gap="3" align="baseline" sx={cardFoot}>
                <strong sx={metric}>{ticks()}</strong>
                <span sx={metricUnit}>ticks · live signal</span>
              </row>
            </column>
          </card>
        </a>
        <a href="/deck" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>Deck</p>
              <strong sx={cardTitle}>SSR, routes, and HMR</strong>
              <p sx={cardBody}>The app host is already wired — layouts, data, and hot reload included.</p>
              <column gap="2" sx={cardFoot}>
                <row gap="2" align="center"><span dot tone="success"></span><span sx={statusLine}>ssr · streaming</span></row>
                <row gap="2" align="center"><span dot tone="primary"></span><span sx={statusLine}>file-based routes</span></row>
                <row gap="2" align="center"><span dot tone="info"></span><span sx={statusLine}>hmr · sub-second</span></row>
              </column>
            </column>
          </card>
        </a>
        <a href="/components" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>Components</p>
              <strong sx={cardTitle}>48 primitives, plus packs</strong>
              <p sx={cardBody}>Forms, overlays, and feedback — themed and accessible by construction.</p>
              <row gap="2" align="center" sx={cardFoot}>
                <badge tone="primary" size="sm">48 L1</badge>
                <badge tone="muted" size="sm" variant="soft">charts</badge>
                <badge tone="muted" size="sm" variant="soft">chat</badge>
                <badge tone="muted" size="sm" variant="soft">editor</badge>
              </row>
            </column>
          </card>
        </a>
        <a href="/ai" sx={cardLink}>
          <card interactive sx={cardFill}>
            <column gap="0" sx={cardCol}>
              <p sx={cardEyebrow}>AI</p>
              <strong sx={cardTitle}>Agents stay inside the language</strong>
              <p sx={cardBody}>The compiler refuses invalid UI — so generated markup cannot drift the system.</p>
              <div sx={[term, cardFoot]}>
                <p><span sx={termAccent}>$</span> sail check app/</p>
                <p>ok · 0 refusals · 214 modules</p>
              </div>
            </column>
          </card>
        </a>
      </grid>
    </else></if>
  </section>

  <section sx={foot}>
    <row justify="between" align="center" sx={footRow}>
      <row gap="6" align="center">
        <a href="/sail/overview" sx={footLink}>Docs</a>
        <a href="https://github.com/get-ship/sail" sx={footLink}>GitHub</a>
      </row>
      <row gap="3" align="center">
        <span sx={diamond}></span>
        <span>SHIP UI · 0.2.0</span>
      </row>
    </row>
  </section>

  <div sx={wordmarkWrap}>
    <p sx={wordmark}>SHIP</p>
  </div>
</column>
