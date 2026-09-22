<!--
  <doc-checkbox> — /components/checkbox
  Shared chrome: sail-component-viewer split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style fieldShell = { size({ w: 100%, max-w: 64 }).pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type(label) }
  style sizeValue = { fg(muted-foreground).type(mono-sm) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style exampleLead = { fg(muted-foreground).type(body-sm) }
  style sectionLabel = { fg(muted-foreground).type(label) }
  style demoRow = { size({ w: 100% }).pad({ y: 2 }) }
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
        <slider bind:value={sizeIdx} min="0" max="3" step="1" size="sm" label="Size" ticks="xs,sm,md,lg"></slider>
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
