<!--
  <doc-progress> — first-class docs host for /components/progress.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-progress.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {

  const PRG_SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const load = signal(62)
  const progIndeterminate = signal(false)
  const progSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), PRG_SIZES.length - 1)
    return PRG_SIZES[i] ?? 'md'
  })
  const progMeta = derived(() => {
    if (progIndeterminate()) return `${progSize()} · indeterminate (no value) · track/fill component.progress.*`
    return `${progSize()} · ${load()}% of 100 · track/fill component.progress.*`
  })
  const entry = DETAILS.progress
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const prgExDeterminate = () =>
    `<progress value={done()} max="100"></progress>\n<p>{done()}% uploaded</p>`
  const prgExSizes = () =>
    `<progress value="50" max="100" size="sm"></progress>\n<progress value="50" max="100"></progress>\n<progress value="50" max="100" size="lg"></progress>`
  const prgExIndeterminate = () =>
    `<progress max="100"></progress>\n<!-- omit value → indeterminate -->`
  const prgExRest = () =>
    `<progress value="64" max="100"></progress>\n<!-- track/fill → component.progress.* -->`
  return {
    sizeIdx, load, progIndeterminate, progSize, progMeta,
    heading, blurb, props, usageCode,
    prgExDeterminate, prgExSizes, prgExIndeterminate, prgExRest,
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
            <if {progIndeterminate()}>
              <progress max="100" size={progSize()} style(fullW)></progress>
            {else =>
              <progress value={load()} max="100" size={progSize()} style(fullW)></progress>}</if>
            <p style(monoXs)>{progMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{progSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>sm</span><span style(tick)>md</span><span style(tick)>lg</span>
            </row>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Value</label>
              <span style(sizeValue)>{progIndeterminate() ? '—' : `${load()}%`}</span>
            </row>
            <slider bind:value={load} min="0" max="100" size="sm" label="Value"></slider>
          </column>
          <column gap="2">
            <label style(knobLabel)>Indeterminate</label>
            <row gap="2" align="center">
              <switch bind:checked={progIndeterminate} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{progIndeterminate() ? 'no value attr' : 'determinate'}</span>
            </row>
            <p style(mutedXs)>Omit <code>value</code> for the native indeterminate sweep. No separate host prop.</p>
          </column>
        </column>
      </row>
    </column>
    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Determinate bar</h3>
        <p style(exampleLead)>Bind <code>value</code> and <code>max</code> — the platform announces completion.</p>
        <column gap="2" style(stageCol)>
          <progress value={load()} max="100"></progress>
          <p style(mutedSm)>{load()}% complete</p>
        </column>
        <code-block code={prgExDeterminate()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)><code>sm</code> · <code>md</code> · <code>lg</code> set track height.</p>
        <column gap="3" style(stageCol)>
          <progress value="50" max="100" size="sm"></progress>
          <progress value="50" max="100"></progress>
          <progress value="50" max="100" size="lg"></progress>
        </column>
        <code-block code={prgExSizes()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Indeterminate (omit value)</h3>
        <p style(exampleLead)>Leave <code>value</code> off — no separate indeterminate prop.</p>
        <column gap="2" style(stageCol)>
          <progress max="100"></progress>
        </column>
        <code-block code={prgExIndeterminate()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>Track/fill from <code>component.progress.*</code>. No tone prop.</p>
        <column gap="2" style(stageCol)>
          <progress value="64" max="100"></progress>
        </column>
        <code-block code={prgExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
