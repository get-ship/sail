<!--
  <doc-ticker> — /components/ticker
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<controller>
import type { Scope } from './doc-ticker.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const duration = signal('normal')
  const reverse = signal(false)

  const entry = DETAILS.ticker
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`duration="${duration()}"`]
    if (reverse()) a.push('reverse')
    return `<ticker ${a.join(' ')} style({ box: w(full).overflow(hidden).padY(2), paint: bg(muted).radius(md) })>
  <p>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
</ticker>`
  })

  const tkExNormal = () =>
    `<ticker duration="normal" style({ box: w(full).overflow(hidden).padY(2), paint: bg(muted).radius(md) })>
  <p>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
</ticker>`
  const tkExReverse = () =>
    `<ticker duration="slow" reverse>
  <p>rightward marquee…&nbsp;</p>
</ticker>`
  const tkExFast = () =>
    `<ticker duration="fast">
  <p>fast 16s loop…&nbsp;</p>
</ticker>`

  return {
    duration,
    reverse,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    tkExNormal,
    tkExReverse,
    tkExFast,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(full).maxW(panel-max-w-md).pad(2).overflow(visible) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
  style ticker = {
    box: w(full).maxW(panel-max-w-md).overflow(hidden).padY(3),
    paint: bg(muted).radius(md),
  }
  style tickerText = {
    box: padX(4),
    paint: fg(muted-foreground),
    text: fontSize(sm).fontFamily(mono).whitespace(nowrap),
  }
</scope>

<column gap="6" style(page)>
  <sail-component-viewer
    name={heading()}
    blurb={blurb()}
    pack="Component primitives"
    layout="split"
    snippet={specimenCode()}
    usage-live
    usage={usageCode()}
    rows={props()}
  >
    <column slot="specimen" gap="3" align="stretch" style(specimen)>
      <if {reverse()}>
        <ticker duration={duration()} reverse style(ticker)>
          <p style(tickerText)>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
        </ticker>
      {else =>
        <ticker duration={duration()} style(ticker)>
          <p style(tickerText)>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
        </ticker>}</if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Duration</label>
        <select size="sm" bind:value={duration} label="Duration">
          <option value="fast">fast</option>
          <option value="normal">normal</option>
          <option value="slow">slow</option>
        </select>
        <p style(mutedXs)>Loop length — fast 16s · normal 24s · slow 32s.</p>
      </column>
      <column gap="2">
        <label style(knobLabel)>Reverse</label>
        <row gap="2" align="center">
          <switch bind:checked={reverse} size="sm" label="Reverse"></switch>
          <span style(sizeValue)>{reverse() ? 'rightward' : 'leftward'}</span>
        </row>
        <p style(mutedXs)>Flips the marquee so it travels the other way.</p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <ticker duration="normal" style(ticker)>
        <p style(tickerText)>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
      </ticker>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Default marquee"
        lead="Host clips the band; content loops horizontally."
        code={tkExNormal()}
      >
        <ticker duration="normal" style(ticker)>
          <p style(tickerText)>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
        </ticker>
      </doc-example>
      <doc-example
        title="Reverse + slow"
        lead='reverse walks the band the other way; duration="slow" lengthens the loop.'
        code={tkExReverse()}
      >
        <ticker duration="slow" reverse style(ticker)>
          <p style(tickerText)>rightward marquee · reduced motion settles ·&nbsp;</p>
        </ticker>
      </doc-example>
      <doc-example
        title="Fast"
        lead='duration="fast" — 16s loop. Colors come from your own styles.'
        code={tkExFast()}
      >
        <ticker duration="fast" style(ticker)>
          <p style(tickerText)>fast loop · paint is yours ·&nbsp;</p>
        </ticker>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
