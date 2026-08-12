<!--
  <doc-label> — /components/label
  Shared chrome: split + snippet copy + usage Code|Live|Split.
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

  const entry = DETAILS.label
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = ['for="email"']
    if (labelSize() !== 'md') a.push(`size="${labelSize()}"`)
    if (labelRequired()) a.push('required')
    return `<label ${a.join(' ')}>Email</label>`
  })

  const exFor = () =>
    `<column gap="1">\n  <label for="email" required>Email</label>\n  <input id="email" placeholder="ada@lovelace.dev" />\n</column>`
  const exSizes = () =>
    `<label for="a" size="sm">sm</label>\n<label for="b">md</label>\n<label for="c" size="lg">lg</label>`
  const exRequired = () => `<label for="email" required>Email</label>`

  return {
    sizeIdx,
    labelRequired,
    labelSize,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exFor,
    exSizes,
    exRequired,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(64).maxW(full).pad(2).overflow(visible) }
  style controlW = { box: w(64).maxW(full) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style tickRow = { box: w(full).padT(1).padX(1_5) }
  style tick = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
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
    <column slot="specimen" gap="1" align="start" style(specimen)>
      <if {labelRequired()}>
        <label for="doc-label-email" required size={labelSize()}>Email</label>
      {else =>
        <label for="doc-label-email" size={labelSize()}>Email</label>}</if>
      <input id="doc-label-email" size={labelSize()} placeholder="ada@lovelace.dev" style(controlW) />
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{labelSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
        <row gap="0" align="center" justify="between" style(tickRow)>
          <span style(tick)>sm</span>
          <span style(tick)>md</span>
          <span style(tick)>lg</span>
        </row>
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={labelRequired} size="sm" /> Required</label>
      </row>
    </column>

    <column slot="usage-preview" gap="1" align="start" style(specimen)>
      <label for="usage-label-email" required>Email</label>
      <input id="usage-label-email" placeholder="ada@lovelace.dev" style(controlW) />
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="for= association"
        lead="Pair for with the control id so click focuses the input."
        code={exFor()}
      >
        <column gap="1" style(controlW)>
          <label for="ex-label-email" required>Email</label>
          <input id="ex-label-email" placeholder="ada@lovelace.dev" />
        </column>
      </doc-example>
      <doc-example
        title="Sizes"
        lead="Match the control size ladder. Default is md."
        code={exSizes()}
      >
        <column gap="3" style(controlW)>
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
      </doc-example>
      <doc-example
        title="Required marker"
        lead="required shows a * next to the label."
        code={exRequired()}
      >
        <column gap="3" style(controlW)>
          <column gap="1">
            <label for="ex-label-req" required>Required field</label>
            <input id="ex-label-req" placeholder="must fill" />
          </column>
          <column gap="1">
            <label for="ex-label-opt">Optional field</label>
            <input id="ex-label-opt" placeholder="can skip" />
          </column>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/field">Field</a>
  </row>
</column>
