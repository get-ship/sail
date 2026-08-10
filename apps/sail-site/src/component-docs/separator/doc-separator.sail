<!--
  <doc-separator> — first-class docs host for /components/separator.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-separator.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {

  const variant = signal('solid')
  const sepOrientation = signal('horizontal')
  const sepTone = signal('')
  const pickSepTone = (t: string): void => sepTone.set(t)
  const clearSepTone = (): void => sepTone.set('')
  const sepMeta = derived(() => {
    const t = sepTone()
    return `${sepOrientation()} · ${variant() || 'solid'} · ${t || 'rest (separator.line)'}`
  })
  const entry = DETAILS.separator
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const sepExBoth = () =>
    `<column gap="2">\n  <p>Section A</p>\n  <separator></separator>\n  <p>Section B</p>\n</column>`
  const sepExVariant = () =>
    `<separator variant="dashed"></separator>\n<separator variant="solid" tone="primary"></separator>`
  const sepExRest = () =>
    `<separator></separator>\n<!-- no tone → component.separator.line -->`
  return {
    variant, sepOrientation, sepTone, sepMeta, pickSepTone, clearSepTone,
    heading, blurb, props, usageCode, sepExBoth, sepExVariant, sepExRest,
  }

})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
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
  style swatchRow = { box: w(full) }
  style swatch = {
    box: w(7).h(7).shrink(0),
    paint: radius(full).cursor(pointer).borderWidth(2).border(transparent).shadow(xs),
    when: {
      selected: { paint: border(foreground).shadow(sm) },
      hover: { paint: opacity(90) },
    },
  }
  style swatchPrimary = { paint: bg(primary) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchMuted = { paint: bg(muted).border(border) }
  style swatchRest = { paint: bg(background).border(border) }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style sepVRow = { box: h(10) }
  style sepStageCol = { box: w(full).maxW(panel-max-w-sm) }

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
          <column gap="4" align="center" style(sepStageCol)>
            <if {sepOrientation() === 'vertical'}>
              <row gap="3" align="center" style(sepVRow)>
                <span style(mutedSm)>A</span>
                <if {sepTone()}>
                  <separator orientation="vertical" variant={variant()} tone={sepTone()}></separator>
                {else =>
                  <separator orientation="vertical" variant={variant()}></separator>}</if>
                <span style(mutedSm)>B</span>
              </row>
            {else =>
              <column gap="2" style(fullW)>
                <p style(mutedSm)>above</p>
                <if {sepTone()}>
                  <separator orientation="horizontal" variant={variant()} tone={sepTone()}></separator>
                {else =>
                  <separator orientation="horizontal" variant={variant()}></separator>}</if>
                <p style(mutedSm)>below</p>
              </column>}</if>
            <p style(monoXs)>{sepMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Orientation</label>
            <select size="sm" bind:value={sepOrientation} label="Orientation">
              <option value="horizontal">horizontal</option>
              <option value="vertical">vertical</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Variant</label>
            <select size="sm" bind:value={variant} label="Variant">
              <option value="solid">solid</option>
              <option value="dashed">dashed</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Tone</label>
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="rest — component.separator.line" placement="top">
                <div label="rest" on:click={clearSepTone} selected={!sepTone() ? 'true' : null} style(swatch, swatchRest)></div>
              </tooltip>
              <tooltip content="muted" placement="top">
                <div label="muted" on:click={() => pickSepTone('muted')} selected={sepTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
              </tooltip>
              <tooltip content="primary" placement="top">
                <div label="primary" on:click={() => pickSepTone('primary')} selected={sepTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickSepTone('destructive')} selected={sepTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
            </row>
            <p style(mutedXs)>Omit tone for themeable <code>separator.line</code>.</p>
          </column>
        </column>
      </row>
    </column>
    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Both orientations</h3>
        <p style(exampleLead)>Horizontal rules sections; vertical sits between inline items.</p>
        <column gap="2" style(sepStageCol)>
          <p style(mutedSm)>Section A</p>
          <separator></separator>
          <p style(mutedSm)>Section B</p>
          <row gap="3" align="center" style(sepVRow)>
            <span style(mutedSm)>Left</span>
            <separator orientation="vertical"></separator>
            <span style(mutedSm)>Right</span>
          </row>
        </column>
        <code-block code={sepExBoth()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Dashed and tone</h3>
        <p style(exampleLead)><code>variant="dashed"</code> draws a border stroke. Tone paints only when set.</p>
        <column gap="3" style(sepStageCol)>
          <separator variant="dashed"></separator>
          <separator variant="solid" tone="primary"></separator>
          <separator variant="dashed" tone="destructive"></separator>
        </column>
        <code-block code={sepExVariant()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Rest line (themeable)</h3>
        <p style(exampleLead)>No <code>tone</code> → <code>component.separator.line</code>.</p>
        <column gap="2" style(sepStageCol)>
          <separator></separator>
          <separator variant="dashed"></separator>
        </column>
        <code-block code={sepExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
