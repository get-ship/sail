<!--
  <doc-select> — /components/select
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen is content-width — not full-bleed.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(56).maxW(full).pad(2).overflow(visible) }
  style controlW = { box: w(56).maxW(full) }
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
import type { Scope } from './doc-select.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const invalid = signal(false)
  const disabled = signal(false)
  const multiple = signal(false)
  const team = signal('design')

  const size = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SIZES.length - 1)
    return SIZES[i] ?? 'md'
  })

  const entry = DETAILS.select
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (size() !== 'md') a.push(`size="${size()}"`)
    if (multiple()) a.push('multiple')
    if (invalid()) a.push('invalid')
    if (disabled()) a.push('disabled')
    if (!multiple()) a.push('bind:value={team}')
    const open = a.length ? `<select ${a.join(' ')}>` : `<select>`
    return `${open}\n  <option value="design">Design</option>\n  <option value="eng">Engineering</option>\n  <option value="research">Research</option>\n</select>`
  })

  const exBind = () =>
    `<select bind:value={team}>\n  <option value="design">Design</option>\n  <option value="eng">Engineering</option>\n</select>`
  const exSizes = () =>
    `<select size="xs">…</select>\n<select size="sm">…</select>\n<select>…</select>\n<select size="lg">…</select>`
  const exInvalid = () =>
    `<select invalid>\n  <option value="">Pick one</option>\n  <option value="a">A</option>\n</select>`

  return {
    sizeIdx,
    size,
    invalid,
    disabled,
    multiple,
    team,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exBind,
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
    <column slot="specimen" gap="3" align="center" style(specimen)>
      <if {multiple()}>
        <select
          size={size()}
          multiple
          invalid={invalid()}
          disabled={disabled()}
          style(controlW)
        >
          <option value="design">Design</option>
          <option value="eng">Engineering</option>
          <option value="research">Research</option>
        </select>
      {else =>
        <select
          size={size()}
          bind:value={team}
          invalid={invalid()}
          disabled={disabled()}
          style(controlW)
        >
          <option value="design">Design</option>
          <option value="eng">Engineering</option>
          <option value="research">Research</option>
        </select>}</if>
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
        <label><checkbox bind:checked={invalid} size="sm" /> Invalid</label>
        <label><checkbox bind:checked={disabled} size="sm" /> Disabled</label>
        <label><checkbox bind:checked={multiple} size="sm" /> Multiple</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(specimen)>
      <select bind:value={team} style(controlW)>
        <option value="design">Design</option>
        <option value="eng">Engineering</option>
        <option value="research">Research</option>
      </select>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="bind:value"
        lead="Options are plain option children."
        code={exBind()}
      >
        <select bind:value={team} style(controlW)>
          <option value="design">Design</option>
          <option value="eng">Engineering</option>
          <option value="research">Research</option>
        </select>
      </doc-example>
      <doc-example
        title="Sizes"
        lead="Default is md."
        code={exSizes()}
      >
        <column gap="2">
          <select size="xs" style(controlW)><option>Ada</option><option>Grace</option></select>
          <select size="sm" style(controlW)><option>Ada</option><option>Grace</option></select>
          <select style(controlW)><option>Ada</option><option>Grace</option></select>
          <select size="lg" style(controlW)><option>Ada</option><option>Grace</option></select>
        </column>
      </doc-example>
      <doc-example
        title="Invalid"
        lead="Mark validation failures with invalid."
        code={exInvalid()}
      >
        <select invalid style(controlW)>
          <option value="">Pick one</option>
          <option value="a">A</option>
        </select>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/combobox">Combobox</a>
  </row>
</column>
