<!--
  Ship UI landing — design 1c KINETIC treatment carrying the REAL landing copy.
  A full-bleed dark marketing home (NOT a docs page):

    · a GIANT rotating headline — one teal word cycles ("Batteries · Components ·
      Styling · State · Motion · Themes") in front of the fixed "included. All of
      them." The cycle is the previous landing's proven WORDS + {#for}+fx:enter
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
import type { ControllerScope } from '@ship/sail-runtime'
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

<scope>
  // minW(0) + overflowX clip on the page root — diagonal tickers use
  // left/right negative inset; without a hard clip the document grows sideways.
  const page = style.w(full).maxW(full).minW(0).overflowX(hidden).overflowY(visible).bg(surface-0)
  const hero = style.position(relative).padX(8).padT(20).padB(16).minW(0).overflowX(hidden)
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
  const bento = style.padX(8).padT(8).padB(12).minW(0).overflowX(hidden)
  const grid = style.w(full).maxW(breakpoint-desktop).mx(auto).minW(0)
  const cardLink = style.display(block).underline(none).minW(0)
  const cardFill = style.h(full).overflow(hidden)
  const cardCol = style.h(full)
  const cardTitle = style.fontSize(lg).fg(foreground).leading(snug)
  const bars = style.h(16).mt(auto)
  const bar = style.w(4).radius(sm).bg(primary)
  const swatchRow = style.mt(auto)
  const swatch = style.w(10).h(10).radius(md)
  const counterRow = style.mt(auto)
  const counter = style.fontFamily(mono).fontSize(display).fg(primary).nums(tabular).leading(none)
  const live = style.fontFamily(mono).fontSize(xs).fg(muted-foreground)
  const deckStatus = style.mt(auto).fontFamily(mono).fontSize(xs).fg(muted-foreground)
  const cardRel = style.h(full).overflow(hidden).position(relative)
  const packBadge = style.position(absolute).bottom(4).right(4).fontFamily(mono).fontSize(xs).fg(muted-foreground)
  const aiCard = style.h(full).overflow(hidden).bg(muted).borderWidth(1).border(primary)
  const term = style.mt(auto).fontFamily(mono).fontSize(sm).bg(surface-0).borderWidth(1).border(border).radius(lg).padX(4).padY(3)
  const cursor = style.display(inline-block).w(1).h(4).bg(primary)
  const foot = style.padX(8).padT(12)
  const footRow = style.w(full).maxW(breakpoint-desktop).mx(auto).padB(6).borderBWidth(1).borderB(border).fontFamily(mono).fontSize(sm).fg(muted-foreground)
  const footLink = style.fg(muted-foreground).when({ hover: style.fg(primary) })
  const diamond = style.display(inline-block).w(3).h(3).rotate(45).bg(primary).radius(sm)
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
          {#for w of word() key={w}}
            <b sx={style.fg(primary)} fx:enter={{ opacity: 'fade' }}>{w}</b>
          {/for}
        </span>
        <span sx={line}>included.</span>
        <span sx={line}>All of them.</span>
      </h1>

      <p sx={lead}>
        One language with the whole interface inside it — built for agents to
        write and humans to review, with no way to drift off the design system.
      </p>

      <row gap="4" justify="center" align="center" sx={ctaRow}>
        <a variant="solid" tone="primary" href="/sail/getting-started" sx={cta} fx:press={{ scale: 'press' }}>Get started</a>
        <a variant="outline" href="/sail/why" sx={cta} fx:press={{ scale: 'press' }}>Why Sail</a>
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
    <grid cols="3" gap="4" sx={grid}>

      <a href="/sail/language/motion" sx={cardLink}>
        <card interactive sx={cardFill}>
          <column gap="4" sx={cardCol}>
            <column gap="1">
              <p sx={eyebrow}>Motion</p>
              <strong sx={cardTitle}>Real physics. Zero choreography code.</strong>
            </column>
            <row gap="2" align="end" sx={bars} data-testid="landing-motion-bars">
              <div data-testid="landing-motion-bar" sx={[bar, style.h(8)]} fx:loop={{ translate-y: 'spring:bouncy' }}></div>
              <div sx={[bar, style.h(14).opacity(60)]} fx:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}></div>
              <div sx={[bar, style.h(11)]} fx:loop={{ translate-y: 'spring:bouncy', stagger: '0.12s' }}></div>
              <div sx={[bar, style.h(16).opacity(60)]} fx:loop={{ translate-y: 'spring:gentle', stagger: '0.12s' }}></div>
              <div sx={[bar, style.h(10)]} fx:loop={{ translate-y: 'spring:bouncy', stagger: '0.12s' }}></div>
            </row>
          </column>
        </card>
      </a>

      <a href="/sail/language/theming" sx={cardLink}>
        <card interactive sx={cardFill}>
          <column gap="4" sx={cardCol}>
            <column gap="1">
              <p sx={eyebrow}>Theming</p>
              <strong sx={cardTitle}>Press a button. The whole site repaints.</strong>
            </column>
            <row gap="2" sx={swatchRow}>
              <div sx={[swatch, style.bg(primary)]} fx:loop={{ scale: 'pulse' }}></div>
              <div sx={[swatch, style.bg(success)]} fx:loop={{ scale: 'pulse', stagger: '0.15s' }}></div>
              <div sx={[swatch, style.bg(warning)]} fx:loop={{ scale: 'pulse', stagger: '0.15s' }}></div>
              <div sx={[swatch, style.bg(info)]} fx:loop={{ scale: 'pulse', stagger: '0.15s' }}></div>
            </row>
          </column>
        </card>
      </a>

      <a href="/sail/language/state" sx={cardLink}>
        <card interactive sx={cardFill}>
          <column gap="4" sx={cardCol}>
            <column gap="1">
              <p sx={eyebrow}>Reactivity</p>
              <strong sx={cardTitle}>State you can watch think.</strong>
            </column>
            <row gap="3" align="baseline" sx={counterRow}>
              <strong sx={counter}>{ticks()}</strong>
              <span sx={live}>signal · live</span>
            </row>
          </column>
        </card>
      </a>

      <a href="/deck" sx={cardLink}>
        <card interactive sx={cardFill}>
          <column gap="4" sx={cardCol}>
            <column gap="1">
              <p sx={eyebrow}>Deck</p>
              <strong sx={cardTitle}>The app layer is already wired.</strong>
            </column>
            <column gap="2" sx={deckStatus}>
              <row gap="2" align="center"><span dot tone="primary" size="sm" pulse></span><span>build 214</span></row>
              <row gap="2" align="center"><span dot tone="success" size="sm" pulse></span><span>deployed</span></row>
              <row gap="2" align="center"><span dot tone="info" size="sm" pulse></span><span>live · ssr</span></row>
            </column>
          </column>
        </card>
      </a>

      <a href="/components" sx={cardLink}>
        <card interactive sx={cardRel}>
          <column gap="4" sx={cardCol}>
            <column gap="1">
              <p sx={eyebrow}>Components</p>
              <strong sx={cardTitle}>The hard parts are already correct.</strong>
            </column>
            <row gap="3" align="center" sx={swatchRow}>
              <switch checked tone="primary"></switch>
              <spinner></spinner>
              <badge tone="success">live</badge>
              <chip>chip</chip>
            </row>
          </column>
          <span sx={packBadge}>48 + packs</span>
        </card>
      </a>

      <a href="/ai" sx={cardLink}>
        <card interactive sx={aiCard}>
          <column gap="4" sx={cardCol}>
            <column gap="1">
              <p sx={eyebrow}>AI</p>
              <strong sx={cardTitle}>Agents can't break what the language won't allow.</strong>
            </column>
            <column gap="1" sx={term}>
              <row gap="2" align="center"><span sx={style.fg(muted-foreground)}>$</span><span sx={style.fg(foreground)}>agent → sail compile</span></row>
              <row gap="1" align="center">
                <span sx={style.fg(primary)}>✓ 0 errors · themed · a11y</span>
                <span sx={cursor} fx:loop={{ scale: 'pulse' }}></span>
              </row>
            </column>
          </column>
        </card>
      </a>

    </grid>
  </section>

  <section sx={foot}>
    <row justify="between" align="center" sx={footRow}>
      <row gap="6" align="center">
        <a href="/sail/overview" sx={footLink}>Docs</a>
        <a href="https://github.com" sx={footLink}>GitHub</a>
        <a href="https://discord.com" sx={footLink}>Discord</a>
      </row>
      <row gap="3" align="center">
        <span sx={diamond}></span>
        <span>SHIP UI · 0.2.0</span>
      </row>
    </row>
  </section>

  <div sx={wordmarkWrap}>
    <p sx={wordmark} fx:scroll={{ scale: 'spring:deliberate' }}>SHIP</p>
  </div>
</column>
