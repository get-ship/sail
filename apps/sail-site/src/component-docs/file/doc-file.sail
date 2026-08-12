<!--
  <doc-file> — /components/file
-->

<controller>
import type { Scope } from './doc-file.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const accept = signal('image/*')
  const multiple = signal(false)
  const disabled = signal(false)

  const size = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SIZES.length - 1)
    return SIZES[i] ?? 'md'
  })

  const entry = DETAILS.file
  const heading = () => entry?.heading ?? '<file>'
  const blurb = () => entry?.blurb ?? ''
  const props = () => entry?.props ?? []
  const usageCode = () => entry?.usage ?? ''

  const specimenCode = derived(() => {
    const a: string[] = []
    if (accept()) a.push(`accept="${accept()}"`)
    if (multiple()) a.push('multiple')
    if (size() !== 'md') a.push(`size="${size()}"`)
    if (disabled()) a.push('disabled')
    return a.length ? `<file ${a.join(' ')} />` : `<file />`
  })

  const exAccept = () => `<file accept="image/*" />\n<file accept=".pdf,application/pdf" multiple />`

  return {
    sizeIdx,
    size,
    accept,
    multiple,
    disabled,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exAccept,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style fieldShell = { box: w(full).maxW(64).pad(2).overflow(visible) }
  style fullW = { box: w(full).overflow(visible) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style tickRow = { box: w(full).padT(1).padX(1_5) }
  style tick = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style exampleSection = { box: w(full).padY(4).overflow(visible) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
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
    <column slot="specimen" gap="3" style(fieldShell)>
      <file
        accept={accept()}
        multiple={multiple()}
        size={size()}
        disabled={disabled()}
        style(fullW)
      />
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Accept</label>
        <input type="text" size="sm" bind:value={accept} placeholder="image/*" />
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
      <row gap="4" align="center">
        <label><checkbox bind:checked={multiple} size="sm" /> Multiple</label>
        <label><checkbox bind:checked={disabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" style(fieldShell)>
      <file accept="image/*" style(fullW) />
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="3" style(exampleSection)>
        <h3 style(exampleH)>Filters</h3>
        <p style(exampleLead)>
          <code>accept</code> limits the system picker. For drag-and-drop, pair with
          <a href="/components/dropzone">dropzone</a>.
        </p>
        <column gap="2" style(fieldShell)>
          <file accept="image/*" style(fullW) />
          <file accept=".pdf,application/pdf" multiple style(fullW) />
        </column>
        <doc-code code={exAccept()} lang="sail"></doc-code>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/dropzone">Dropzone</a>
  </row>
</column>
