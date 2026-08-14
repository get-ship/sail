<!--
  <doc-dropzone> — /components/dropzone
  The <dropzone> element. File's variant="drop" is the same look on <file>.
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: full, max-w: panel-max-w-sm }).overflow({ all: visible }) }
  style fullW = { size({ w: full }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-dropzone.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const accept = signal('image/*')
  const multiple = signal(true)
  const disabled = signal(false)

  const entry = DETAILS.dropzone
  const heading = () => '<dropzone>'
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (accept()) a.push(`accept="${accept()}"`)
    if (multiple()) a.push('multiple')
    if (disabled()) a.push('disabled')
    return a.length ? `<dropzone ${a.join(' ')} />` : `<dropzone />`
  })

  const exImages = () => `<dropzone accept="image/*" multiple />`
  const exPdf = () => `<dropzone accept=".pdf,application/pdf" />`
  const exFileVariant = () => `<file variant="drop" accept="image/*" multiple />`

  return {
    accept,
    multiple,
    disabled,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exImages,
    exPdf,
    exFileVariant,
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
      <dropzone
        accept={accept()}
        multiple={multiple()}
        disabled={disabled()}
        style(fullW)
      />
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Accept</label>
        <input type="text" size="sm" bind:value={accept} placeholder="image/*" />
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={multiple} size="sm" /> Multiple</label>
        <label><checkbox bind:checked={disabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" style(specimen)>
      <dropzone accept="image/*" multiple style(fullW) />
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Images"
        lead="Drag images onto the surface, or click to browse."
        code={exImages()}
      >
        <dropzone accept="image/*" multiple style(fullW) />
      </doc-example>
      <doc-example
        title="PDFs only"
        lead="Same surface, tighter accept filter."
        code={exPdf()}
      >
        <dropzone accept=".pdf,application/pdf" style(fullW) />
      </doc-example>
      <doc-example
        title="File variant"
        lead='Need it on the file tag? variant="drop" is the same surface.'
        code={exFileVariant()}
      >
        <file variant="drop" accept="image/*" multiple style(fullW) />
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/file">File</a>
  </row>
</column>
