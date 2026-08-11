<!--
  <doc-label> — first-class docs host for /components/label.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-label.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const LABEL_SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const labelRequired = signal(true)
  const labelSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), LABEL_SIZES.length - 1)
    return LABEL_SIZES[i] ?? 'md'
  })
  const labelMeta = derived(() => {
    return `${labelSize()} · required ${labelRequired() ? 'on' : 'off'} · component.label.fg + required`
  })
  const entry = DETAILS.label
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const lblExFor = () =>
    `<column gap="1">\n  <label for="email" required>Email</label>\n  <input id="email" placeholder="ada@lovelace.dev" />\n</column>`
  const lblExSizes = () =>
    `<label for="a" size="sm">sm</label>\n<label for="b">md</label>\n<label for="c" size="lg">lg</label>`
  const lblExRequired = () =>
    `<label for="email" required>Email</label>\n<!-- * → component.label.required; omit required for optional -->`
  const lblExRest = () =>
    `<label for="name">Display name</label>\n<!-- ink → component.label.fg; weight → typography.label.weight -->`
  return {
    sizeIdx, labelRequired, labelSize, labelMeta,
    heading, blurb, props, usageCode,
    lblExFor, lblExSizes, lblExRequired, lblExRest,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
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
            <column gap="1" style(stageCol)>
              <if {labelRequired()}>
                <label for="doc-label-email" required size={labelSize()}>Email</label>
              {else =>
                <label for="doc-label-email" size={labelSize()}>Email</label>}</if>
              <input id="doc-label-email" size={labelSize()} placeholder="ada@lovelace.dev" />
            </column>
            <p style(monoXs)>{labelMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{labelSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>sm</span><span style(tick)>md</span><span style(tick)>lg</span>
            </row>
            <p style(mutedXs)>
              Match the control size ladder. Default <code>md</code>.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Required</label>
            <row gap="2" align="center">
              <switch bind:checked={labelRequired} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{labelRequired() ? 'marker on' : 'optional'}</span>
            </row>
            <p style(mutedXs)>
              <code>required</code> shows the <code>*</code> via
              <code>group-required</code> · colour from
              <code>component.label.required</code>.
            </p>
          </column>
          <column gap="2">
            <p style(mutedXs)>
              Host ink from <code>component.label.fg</code>; weight from
              <code>typography.label.weight</code>. No tone prop.
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>for= association</h3>
        <p style(exampleLead)>
          Pair <code>for</code> with the control <code>id</code> so click focuses the input.
        </p>
        <column gap="1" style(stageCol)>
          <label for="ex-label-email" required>Email</label>
          <input id="ex-label-email" placeholder="ada@lovelace.dev" />
        </column>
        <code-block code={lblExFor()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)>
          <code>sm</code> · <code>md</code> · <code>lg</code> match the control type ladder.
        </p>
        <column gap="3" style(stageCol)>
          <column gap="1">
            <label for="ex-label-sm" size="sm">Small</label>
            <input id="ex-label-sm" size="sm" placeholder="sm" />
          </column>
          <column gap="1">
            <label for="ex-label-md">Medium</label>
            <input id="ex-label-md" placeholder="md" />
          </column>
          <column gap="1">
            <label for="ex-label-lg" size="lg">Large</label>
            <input id="ex-label-lg" size="lg" placeholder="lg" />
          </column>
        </column>
        <code-block code={lblExSizes()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Required marker</h3>
        <p style(exampleLead)>
          <code>required</code> reveals the <code>*</code> (aria-hidden) via
          <code>group-required</code>. Colour from <code>component.label.required</code>.
        </p>
        <column gap="3" style(stageCol)>
          <column gap="1">
            <label for="ex-label-req" required>Required field</label>
            <input id="ex-label-req" placeholder="must fill" />
          </column>
          <column gap="1">
            <label for="ex-label-opt">Optional field</label>
            <input id="ex-label-opt" placeholder="can skip" />
          </column>
        </column>
        <code-block code={lblExRequired()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Host ink from <code>component.label.fg</code>
          (<code>--label-foreground</code>). No tone prop — override the leaf in Studio.
        </p>
        <column gap="1" style(stageCol)>
          <label for="ex-label-rest">Display name</label>
          <input id="ex-label-rest" placeholder="Ada Lovelace" />
        </column>
        <code-block code={lblExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
