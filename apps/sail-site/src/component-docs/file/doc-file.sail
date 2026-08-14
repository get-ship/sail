<!--
  <doc-file> — /components/file
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style fieldShell = { size({ w: full, max-w: 64 }).pad(2).overflow({ all: visible }) }
  style fullW = { size({ w: full }).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style tickRow = { size({ w: full }).pad({ x: 1_5, t: 1 }) }
  style tick = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

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
  const exDrop = () => `<file variant="drop" accept="image/*" multiple />`

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
    exDrop,
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
        <slider bind:value={sizeIdx} min="0" max="3" step="1" size="sm" label="Size" ticks="xs,sm,md,lg"></slider>
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
      <doc-example
        title="Filters"
        lead="accept limits the system picker."
        code={exAccept()}
      >
        <column gap="2" style(fieldShell)>
          <file accept="image/*" style(fullW) />
          <file accept=".pdf,application/pdf" multiple style(fullW) />
        </column>
      </doc-example>
      <doc-example
        title="Drop surface"
        lead='variant="drop" is the dashed target. Same accept and multiple as the picker.'
        code={exDrop()}
      >
        <file variant="drop" accept="image/*" multiple style(fullW) />
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
  </row>
</column>
