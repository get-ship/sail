<!--
  <doc-checkbox> — /components/checkbox
  Shared chrome: sail-component-viewer split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style fieldShell = { box: w(full).maxW(64).pad(2).overflow(visible) }
  style fullW = { box: w(full) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style tickRow = { box: w(full).padT(1).padX(1_5) }
  style tick = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
  style demoRow = { box: w(full).padY(2) }
</scope>

<controller>
import type { Scope } from './doc-checkbox.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const checked = signal(true)
  const boxDisabled = signal(false)
  const boxInvalid = signal(false)

  const size = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SIZES.length - 1)
    return SIZES[i] ?? 'md'
  })

  const entry = DETAILS.checkbox
  const heading = () => entry?.heading ?? '<checkbox>'
  const blurb = () => entry?.blurb ?? ''
  const props = () => entry?.props ?? []
  const usageCode = () => entry?.usage ?? ''

  const specimenCode = derived(() => {
    const a: string[] = ['bind:checked={on}']
    if (size() !== 'md') a.push(`size="${size()}"`)
    if (boxInvalid()) a.push('invalid')
    if (boxDisabled()) a.push('disabled')
    return `<label>\n  <checkbox ${a.join(' ')} />\n  I agree\n</label>`
  })

  const exSizes = () =>
    `<label><checkbox size="xs" checked /> xs</label>\n<label><checkbox size="sm" checked /> sm</label>\n<label><checkbox checked /> md</label>\n<label><checkbox size="lg" checked /> lg</label>`
  const exInvalid = () =>
    `<label><checkbox invalid /> Required</label>`

  return {
    sizeIdx,
    size,
    checked,
    boxDisabled,
    boxInvalid,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exSizes,
    exInvalid,
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
    <column slot="specimen" gap="3" align="start" style(fieldShell)>
      <label>
        <checkbox
          bind:checked={checked}
          size={size()}
          invalid={boxInvalid()}
          disabled={boxDisabled()}
        />
        I agree to the terms
      </label>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{size()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="3" step="1" size="sm" label="Size"></slider>
        <row gap="0" align="center" justify="between" style(tickRow)>
          <span style(tick)>xs</span>
          <span style(tick)>sm</span>
          <span style(tick)>md</span>
          <span style(tick)>lg</span>
        </row>
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={checked} size="sm" /> Checked</label>
        <label><checkbox bind:checked={boxInvalid} size="sm" /> Invalid</label>
        <label><checkbox bind:checked={boxDisabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" style(fieldShell)>
      <label>
        <checkbox bind:checked={checked} size="md" />
        I agree to the terms
      </label>
      <p style(exampleLead)>{checked() ? 'Accepted' : 'Please accept'}</p>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Sizes"
        lead="Default is md."
        code={exSizes()}
      >
        <row gap="4" align="center" style(demoRow)>
          <label><checkbox size="xs" checked /> xs</label>
          <label><checkbox size="sm" checked /> sm</label>
          <label><checkbox checked /> md</label>
          <label><checkbox size="lg" checked /> lg</label>
        </row>
      </doc-example>

      <doc-example
        title="Invalid"
        lead="Mark validation failures with invalid."
        code={exInvalid()}
      >
        <row gap="3" align="center" style(demoRow)>
          <label><checkbox invalid /> Required</label>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/switch">Switch</a>
  </row>
</column>
