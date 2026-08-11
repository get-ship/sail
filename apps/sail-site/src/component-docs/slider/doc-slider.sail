<!--
  <doc-slider> — first-class docs host for /components/slider.
  Specimen left, knobs right + examples. Single thumb only.
-->

<controller>
import type { Scope } from './doc-slider.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SLIDER_SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const load = signal(60)
  const sliderDisabled = signal(false)
  const sliderSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SLIDER_SIZES.length - 1)
    return SLIDER_SIZES[i] ?? 'md'
  })
  const sliderMeta = derived(() => {
    const d = sliderDisabled() ? 'disabled' : 'interactive'
    return `${sliderSize()} · ${load()} of 0–100 · ${d} · track/fill component.slider.*`
  })
  const entry = DETAILS.slider
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const sldExBind = () =>
    `// volume = signal(48)\n<column gap="2">\n  <row justify="between"><span>Volume</span><span>{volume()}%</span></row>\n  <slider bind:value={volume} min="0" max="100"></slider>\n</column>`
  const sldExSizes = () =>
    `<slider value="50" size="sm"></slider>\n<slider value="50"></slider>\n<slider value="50" size="lg"></slider>`
  const sldExDisabled = () =>
    `<slider value="40" disabled></slider>\n<!-- native freeze + opacity; fill still shows value -->`
  const sldExRest = () =>
    `<slider value="62" min="0" max="100"></slider>\n<!-- track/fill/border → component.slider.*; single thumb only -->`
  return {
    sizeIdx, load, sliderDisabled, sliderSize, sliderMeta,
    heading, blurb, props, usageCode,
    sldExBind, sldExSizes, sldExDisabled, sldExRest,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style fullW = { box: w(full) }
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
  style tickRow = { box: w(full).padT(1) }
  style tick = {
    box: grow(1).basis(0).minW(0),
    paint: fg(muted-foreground),
    text: fontSize(xs).fontFamily(mono).textAlign(center),
  }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
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
          <column gap="4" align="center" style(stageCol)>
            <if {sliderDisabled()}>
              <slider value={load()} min="0" max="100" step="1" size={sliderSize()} disabled style(fullW)></slider>
            {else =>
              <slider bind:value={load} min="0" max="100" step="1" size={sliderSize()} style(fullW)></slider>}</if>
            <p style(monoXs)>{sliderMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{sliderSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>sm</span><span style(tick)>md</span><span style(tick)>lg</span>
            </row>
            <p style(mutedXs)>
              Scales track height + thumb diameter. Default <code>md</code>.
            </p>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Value</label>
              <span style(sizeValue)>{load()}</span>
            </row>
            <slider bind:value={load} min="0" max="100" size="sm" label="Value"></slider>
            <p style(mutedXs)>
              Drag the specimen or this control. Stage fixes min=0 max=100 step=1.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Disabled</label>
            <row gap="2" align="center">
              <switch bind:checked={sliderDisabled} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{sliderDisabled() ? 'not-allowed' : 'interactive'}</span>
            </row>
            <p style(mutedXs)>
              Native <code>disabled</code> freezes drag/keyboard; PE applies opacity + not-allowed cursor.
            </p>
          </column>
          <column gap="2">
            <p style(mutedXs)>
              Paint from <code>component.slider.track</code> / <code>fill</code> /
              <code>track-border</code>. Single thumb only — no dual-value range.
            </p>
          </column>
        </column>
      </row>
    </column>
    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>bind:value</h3>
        <p style(exampleLead)>
          A native <code>type=range</code> control. Pair with adjacent copy for the readout.
        </p>
        <column gap="2" style(stageCol)>
          <row justify="between" align="center">
            <span style(mutedSm)>Volume</span>
            <span style(monoXs)>{load()}%</span>
          </row>
          <slider bind:value={load} min="0" max="100" style(fullW)></slider>
        </column>
        <code-block code={sldExBind()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)>
          <code>sm</code> · <code>md</code> · <code>lg</code> scale track height and thumb diameter.
        </p>
        <column gap="3" style(stageCol)>
          <slider value="50" size="sm" style(fullW)></slider>
          <slider value="50" style(fullW)></slider>
          <slider value="50" size="lg" style(fullW)></slider>
        </column>
        <code-block code={sldExSizes()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Disabled</h3>
        <p style(exampleLead)>
          Native <code>disabled</code> freezes interaction. The filled track still shows the current value.
        </p>
        <column gap="2" style(stageCol)>
          <slider value="40" disabled style(fullW)></slider>
        </column>
        <code-block code={sldExDisabled()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Track / fill / track-border from <code>component.slider.*</code>. Single thumb only.
        </p>
        <column gap="2" style(stageCol)>
          <slider value="62" min="0" max="100" style(fullW)></slider>
        </column>
        <code-block code={sldExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
