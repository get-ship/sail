<!--
  <doc-meter> — first-class docs host for /components/meter.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

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
  const meterMeta = derived(() => {
    return `${meterSize()} · value ${meterVal()} · ${meterBand()} · component.meter.*`
  })
  const entry = DETAILS.meter
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const mtrExMeasure = () =>
    `<column gap="2">\n  <p>Disk free — 72%</p>\n  <meter value="72" min="0" max="100" low="33" high="66" optimum="100"></meter>\n</column>`
  const mtrExSizes = () =>
    `<meter value="0.55" size="sm"></meter>\n<meter value="0.55"></meter>\n<meter value="0.55" size="lg"></meter>`
  const mtrExBands = () =>
    `<meter value="20" min="0" max="100" low="33" high="66" optimum="100"></meter>\n<meter value="50" min="0" max="100" low="33" high="66" optimum="100"></meter>\n<meter value="85" min="0" max="100" low="33" high="66" optimum="100"></meter>`
  const mtrExRest = () =>
    `<meter value="0.55"></meter>\n<!-- track → component.meter.track -->`
  return {
    sizeIdx, meterVal, meterSize, meterBand, meterMeta,
    heading, blurb, props, usageCode, mtrExMeasure, mtrExSizes, mtrExBands, mtrExRest,
  }

})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageColMd = { box: w(panel-max-w-md).maxW(full) }
  style meterCol = { box: w(panel-max-w-sm) }
  style fullW = { box: w(full) }
  style grow1 = { box: grow(1) }
  style stageSplit = { box: w(full).minW(0).minH(64) }
  style stageCanvas = { box: grow(1).basis(0).minW(0).minH(64).pad(8) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style swatchRow = { box: w(full) }
  style tickRow = { box: w(full).padT(1) }
  style tick = {
    box: grow(1).basis(0).minW(0),
    paint: fg(muted-foreground),
    text: fontSize(xs).fontFamily(mono).textAlign(center),
  }
  style swatch = {
    box: w(7).h(7).shrink(0),
    paint: radius(full).cursor(pointer).borderWidth(2).border(transparent).shadow(xs),
    when: {
      selected: { paint: border(foreground).shadow(sm) },
      hover: { paint: opacity(90) },
    },
  }
  style swatchPrimary = { paint: bg(primary) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted).border(border) }
  style swatchInfo = { paint: bg(info) }
  style swatchRest = { paint: bg(background).border(border) }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style sepVRow = { box: h(10) }
  style sepStageCol = { box: w(full).maxW(panel-max-w-sm) }
  style skelStageCol = { box: w(full).maxW(panel-max-w-sm) }
  style skelBlock = { box: w(full).maxW(panel-max-w-sm) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }
</scope>

<column gap="4" style(page)>
  <sail-component-viewer
    name={heading()}
    blurb={blurb()}
    pack="Component primitives"
    variant-attr=""
    usage={usageCode()}
    variants={[]}
    rows={props()}
  >
    <column slot="stage" gap="4" align="stretch" style({ box: w(full).minW(0) })>
      <row nowrap gap="0" align="stretch" style(stageSplit)>
        <center style(stageCanvas)>
          <column gap="4" align="center" style(meterCol)>
            <meter value={meterVal()} min="0" max="100" low="33" high="66" optimum="100" size={meterSize()}></meter>
            <p style(monoXs)>{meterMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Value</label>
              <span style(sizeValue)>{meterVal()} · {meterBand()}</span>
            </row>
            <slider bind:value={meterVal} min="0" max="100" size="sm" label="Value"></slider>
            <p style(mutedXs)>Fixed thresholds low=33 · high=66 · optimum=100.</p>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{meterSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>sm</span><span style(tick)>md</span><span style(tick)>lg</span>
            </row>
          </column>
        </column>
      </row>
    </column>
    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Labeled measurement</h3>
        <p style(exampleLead)>A gauge for a quantity in range — not task completion.</p>
        <column gap="2" style(meterCol)>
          <p style(mutedSm)>Disk free — 72%</p>
          <meter value="72" min="0" max="100" low="33" high="66" optimum="100"></meter>
        </column>
        <code-block code={mtrExMeasure()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)><code>sm</code> · <code>md</code> · <code>lg</code> scale bar height.</p>
        <column gap="3" style(meterCol)>
          <meter value="0.55" size="sm"></meter>
          <meter value="0.55"></meter>
          <meter value="0.55" size="lg"></meter>
        </column>
        <code-block code={mtrExSizes()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Optimum bands</h3>
        <p style(exampleLead)>low/high/optimum drive the three fill regions.</p>
        <column gap="3" style(meterCol)>
          <column gap="1">
            <p style(monoXs)>value 20 · even-less-good</p>
            <meter value="20" min="0" max="100" low="33" high="66" optimum="100"></meter>
          </column>
          <column gap="1">
            <p style(monoXs)>value 50 · suboptimum</p>
            <meter value="50" min="0" max="100" low="33" high="66" optimum="100"></meter>
          </column>
          <column gap="1">
            <p style(monoXs)>value 85 · optimum</p>
            <meter value="85" min="0" max="100" low="33" high="66" optimum="100"></meter>
          </column>
        </column>
        <code-block code={mtrExBands()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>Track from <code>component.meter.track</code>; bands from optimum leaves.</p>
        <column gap="2" style(meterCol)>
          <meter value="0.55"></meter>
        </column>
        <code-block code={mtrExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
