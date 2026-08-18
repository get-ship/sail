<!--
  <doc-textarea> — /components/textarea
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 64, max-w: 100% }).pad(2).overflow({ all: visible }) }
  style controlW = { size({ w: 100% }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-textarea.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const size = signal('md')
  const invalid = signal(false)
  const disabled = signal(false)
  const placeholder = signal('Tell us about yourself')
  const rows = signal('3')
  const bio = signal('')

  const entry = DETAILS.textarea
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`rows="${rows()}"`, 'bind:value={bio}']
    if (size() !== 'md') a.push(`size="${size()}"`)
    if (placeholder()) a.push(`placeholder="${placeholder()}"`)
    if (invalid()) a.push('invalid')
    if (disabled()) a.push('disabled')
    return `<textarea ${a.join(' ')} />`
  })

  const exBind = () =>
    `<textarea rows="3" placeholder="Notes" bind:value={notes} />`
  const exSizes = () =>
    `<textarea size="xs" rows="2" placeholder="xs" />\n<textarea size="sm" rows="2" placeholder="sm" />\n<textarea rows="2" placeholder="md" />\n<textarea size="lg" rows="2" placeholder="lg" />`

  return {
    size,
    invalid,
    disabled,
    placeholder,
    rows,
    bio,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exBind,
    exSizes,
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
    <column slot="specimen" gap="3" style(specimen)>
      <textarea
        size={size()}
        rows={rows()}
        placeholder={placeholder()}
        bind:value={bio}
        invalid={invalid()}
        disabled={disabled()}
        style(controlW)
     />
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Placeholder</label>
        <input type="text" size="sm" bind:value={placeholder} />
      </column>
      <column gap="2">
        <label style(knobLabel)>Size</label>
        <select size="sm" bind:value={size} label="Size">
          <option value="xs">xs</option>
          <option value="sm">sm</option>
          <option value="md">md</option>
          <option value="lg">lg</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Rows</label>
        <select size="sm" bind:value={rows} label="Rows">
          <option value="2">2</option>
          <option value="3">3</option>
          <option value="4">4</option>
          <option value="6">6</option>
          <option value="8">8</option>
        </select>
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={invalid} size="sm" /> Invalid</label>
        <label><checkbox bind:checked={disabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" style(specimen)>
      <textarea rows="3" placeholder="Notes" bind:value={bio} style(controlW) />
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="bind:value"
        lead="Two-way binding keeps the field and your signal in lockstep."
        code={exBind()}
      >
        <textarea rows="3" placeholder="Notes" bind:value={bio} style(specimen) />
      </doc-example>
      <doc-example
        title="Sizes"
        lead="Default is md."
        code={exSizes()}
      >
        <column gap="2" style(specimen)>
          <textarea size="xs" rows="2" placeholder="xs" />
          <textarea size="sm" rows="2" placeholder="sm" />
          <textarea rows="2" placeholder="md" />
          <textarea size="lg" rows="2" placeholder="lg" />
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/input">Input</a>
  </row>
</column>
