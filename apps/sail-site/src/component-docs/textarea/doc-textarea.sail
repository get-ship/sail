<!--
  <doc-textarea> — /components/textarea
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(64).maxW(full).pad(2).overflow(visible) }
  style controlW = { box: w(full) }
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

<controller>
import type { Scope } from './doc-textarea.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const invalid = signal(false)
  const disabled = signal(false)
  const placeholder = signal('Tell us about yourself')
  const rowsIdx = signal(3)
  const bio = signal('')

  const size = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SIZES.length - 1)
    return SIZES[i] ?? 'md'
  })
  const rows = derived(() => {
    const n = Math.round(Number(rowsIdx()) || 3)
    return Math.min(12, Math.max(1, n))
  })

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
    return `<textarea ${a.join(' ')}></textarea>`
  })

  const exBind = () =>
    `<textarea rows="3" placeholder="Notes" bind:value={notes}></textarea>`
  const exSizes = () =>
    `<textarea size="xs" rows="2" placeholder="xs"></textarea>\n<textarea size="sm" rows="2" placeholder="sm"></textarea>\n<textarea rows="2" placeholder="md"></textarea>\n<textarea size="lg" rows="2" placeholder="lg"></textarea>`

  return {
    sizeIdx,
    size,
    invalid,
    disabled,
    placeholder,
    rowsIdx,
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
      ></textarea>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Placeholder</label>
        <input type="text" size="sm" bind:value={placeholder} />
      </column>
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
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Rows</label>
          <span style(sizeValue)>{rows()}</span>
        </row>
        <slider bind:value={rowsIdx} min="1" max="8" step="1" size="sm" label="Rows"></slider>
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={invalid} size="sm" /> Invalid</label>
        <label><checkbox bind:checked={disabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" style(specimen)>
      <textarea rows="3" placeholder="Notes" bind:value={bio} style(controlW)></textarea>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="bind:value"
        lead="Two-way binding keeps the field and your signal in lockstep."
        code={exBind()}
      >
        <textarea rows="3" placeholder="Notes" bind:value={bio} style(specimen)></textarea>
      </doc-example>
      <doc-example
        title="Sizes"
        lead="Default is md."
        code={exSizes()}
      >
        <column gap="2" style(specimen)>
          <textarea size="xs" rows="2" placeholder="xs"></textarea>
          <textarea size="sm" rows="2" placeholder="sm"></textarea>
          <textarea rows="2" placeholder="md"></textarea>
          <textarea size="lg" rows="2" placeholder="lg"></textarea>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/input">Input</a>
  </row>
</column>
