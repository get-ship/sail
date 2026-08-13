<!--
  <doc-ticker> — /components/ticker
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style specimen = { box: size({ w: full, max-w: panel-max-w-md }).space({ pad: 2 }).overflow({ all: visible }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style sizeValue = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style mutedXs = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
  style ticker = {
    box: size({ w: full, max-w: panel-max-w-md }).overflow({ all: hidden }).space({ pad-y: 3 }),
    paint: fill({ color: muted }).radius(md),
}
  style tickerText = {
    box: space({ pad-x: 4 }),
    paint: ink({ color: muted-foreground }),
    text: type({ family: mono, size: sm, wrap: nowrap }),
}
</scope>

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
    return `<ticker ${a.join(' ')} style({ box: size({ w: full }).overflow({ all: hidden }).space({ pad-y: 2 }), paint: fill({ color: muted }).radius(md) })>
  <p>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
</ticker>`
  })

  const tkExNormal = () =>
    `<ticker duration="normal" style({ box: size({ w: full }).overflow({ all: hidden }).space({ pad-y: 2 }), paint: fill({ color: muted }).radius(md) })>
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
