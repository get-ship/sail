<!--
  <doc-meter> — /components/meter
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style specimen = { box: size({ w: 64, max-w: full }).space({ pad: 2 }).overflow({ all: visible }) }
  style controlW = { box: size({ w: full }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style monoXs = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style sizeValue = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style tickRow = { box: size({ w: full }).space({ pad-x: 1_5, pad-t: 1 }) }
  style tick = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
</scope>

<controller>
import type { Scope } from './doc-meter.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const METER_SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const meterVal = signal(55)

  const meterSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), METER_SIZES.length - 1)
    return METER_SIZES[i] ?? 'md'
  })
  const meterBand = derived(() => {
    const v = Number(meterVal()) || 0
    if (v >= 66) return 'optimum'
    if (v >= 33) return 'suboptimum'
    return 'even-less-good'
  })

  const entry = DETAILS.meter
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [
      `value="${meterVal()}"`,
      'min="0"',
      'max="100"',
      'low="33"',
      'high="66"',
      'optimum="100"',
    ]
    if (meterSize() !== 'md') a.push(`size="${meterSize()}"`)
    return `<meter ${a.join(' ')}></meter>`
  })

  const mtrExMeasure = () =>
    `<column gap="2">\n  <p>Disk free — 72%</p>\n  <meter value="72" min="0" max="100" low="33" high="66" optimum="100"></meter>\n</column>`
  const mtrExSizes = () =>
    `<meter value="0.55" size="sm"></meter>\n<meter value="0.55"></meter>\n<meter value="0.55" size="lg"></meter>`
  const mtrExBands = () =>
    `<meter value="20" min="0" max="100" low="33" high="66" optimum="100"></meter>\n<meter value="50" min="0" max="100" low="33" high="66" optimum="100"></meter>\n<meter value="85" min="0" max="100" low="33" high="66" optimum="100"></meter>`

  return {
    sizeIdx,
    meterVal,
    meterSize,
    meterBand,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    mtrExMeasure,
    mtrExSizes,
    mtrExBands,
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
      <meter
        value={meterVal()}
        min="0"
        max="100"
        low="33"
        high="66"
        optimum="100"
        size={meterSize()}
        style(controlW)
      ></meter>
      <row justify="between" align="center">
        <span style(sizeValue)>{meterVal()}</span>
        <span style(sizeValue)>{meterBand()}</span>
      </row>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Value</label>
          <span style(sizeValue)>{meterVal()} · {meterBand()}</span>
        </row>
        <slider bind:value={meterVal} min="0" max="100" size="sm" label="Value"></slider>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{meterSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size" ticks="sm,md,lg"></slider>
      </column>
    </column>

    <column slot="usage-preview" gap="3" style(specimen)>
      <p style(mutedSm)>Disk free — 72%</p>
      <meter
        value="72"
        min="0"
        max="100"
        low="33"
        high="66"
        optimum="100"
        style(controlW)
      ></meter>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Labeled measurement"
        lead="A gauge for a quantity in range — not task completion."
        code={mtrExMeasure()}
      >
        <column gap="2" style(specimen)>
          <p style(mutedSm)>Disk free — 72%</p>
          <meter
            value="72"
            min="0"
            max="100"
            low="33"
            high="66"
            optimum="100"
            style(controlW)
          ></meter>
        </column>
      </doc-example>
      <doc-example title="Sizes" lead="Default is md." code={mtrExSizes()}>
        <column gap="3" style(specimen)>
          <meter value="0.55" size="sm" style(controlW)></meter>
          <meter value="0.55" style(controlW)></meter>
          <meter value="0.55" size="lg" style(controlW)></meter>
        </column>
      </doc-example>
      <doc-example
        title="Optimum bands"
        lead="low, high, and optimum drive the three fill regions."
        code={mtrExBands()}
      >
        <column gap="3" style(specimen)>
          <column gap="1">
            <p style(monoXs)>value 20 · even-less-good</p>
            <meter
              value="20"
              min="0"
              max="100"
              low="33"
              high="66"
              optimum="100"
              style(controlW)
            ></meter>
          </column>
          <column gap="1">
            <p style(monoXs)>value 50 · suboptimum</p>
            <meter
              value="50"
              min="0"
              max="100"
              low="33"
              high="66"
              optimum="100"
              style(controlW)
            ></meter>
          </column>
          <column gap="1">
            <p style(monoXs)>value 85 · optimum</p>
            <meter
              value="85"
              min="0"
              max="100"
              low="33"
              high="66"
              optimum="100"
              style(controlW)
            ></meter>
          </column>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
