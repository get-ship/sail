<!--
  <doc-spinner> — /components/spinner
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 100% }).pad(2).overflow({ all: visible }) }
  style mutedSm = { fg(muted-foreground).type({ size: sm }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchPrimary = { bg(primary) }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDanger = { bg(danger) }
  style swatchAccent = { bg(accent) }
  style swatchInfo = { bg(info) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style demoRow = { size({ w: 100% }).pad({ y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-spinner.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const spinnerTone = signal('info')
  const spinnerSize = signal('md')

  const pickSpinnerTone = (t: string): void => spinnerTone.set(t)

  const entry = DETAILS.spinner
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (spinnerSize() !== 'md') a.push(`size="${spinnerSize()}"`)
    a.push(`tone="${spinnerTone()}"`)
    return `<spinner ${a.join(' ')}></spinner>`
  })

  const spnExInline = () =>
    `<row gap="2" align="center">\n  <spinner size="sm" tone="info"></spinner>\n  <p>fetching…</p>\n</row>`
  const spnExSizes = () =>
    `<spinner size="xs" tone="info"></spinner>\n<spinner size="sm" tone="info"></spinner>\n<spinner tone="info"></spinner>\n<spinner size="lg" tone="info"></spinner>`
  const spnExDefault = () => `<spinner tone="info"></spinner>`

  return {
    spinnerTone,
    spinnerSize,
    pickSpinnerTone,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    spnExInline,
    spnExSizes,
    spnExDefault,
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
      <row gap="2" align="center">
        <spinner
          size={spinnerSize()}
          tone={spinnerTone()}
        ></spinner>
      </row>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Size</label>
        <select size="sm" bind:value={spinnerSize} label="Size">
          <option value="xs">xs</option>
          <option value="sm">sm</option>
          <option value="md">md</option>
          <option value="lg">lg</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="primary"
            selected={spinnerTone() === 'primary' ? 'true' : null}
            on:click={() => pickSpinnerTone('primary')}
            style(swatch, swatchPrimary)
          ></div>
          <div
            title="info"
            selected={spinnerTone() === 'info' ? 'true' : null}
            on:click={() => pickSpinnerTone('info')}
            style(swatch, swatchInfo)
          ></div>
          <div
            title="success"
            selected={spinnerTone() === 'success' ? 'true' : null}
            on:click={() => pickSpinnerTone('success')}
            style(swatch, swatchSuccess)
          ></div>
          <div
            title="warning"
            selected={spinnerTone() === 'warning' ? 'true' : null}
            on:click={() => pickSpinnerTone('warning')}
            style(swatch, swatchWarning)
          ></div>
          <div
            title="danger"
            selected={spinnerTone() === 'danger' ? 'true' : null}
            on:click={() => pickSpinnerTone('danger')}
            style(swatch, swatchDanger)
          ></div>
          <div
            title="accent"
            selected={spinnerTone() === 'accent' ? 'true' : null}
            on:click={() => pickSpinnerTone('accent')}
            style(swatch, swatchAccent)
          ></div>
        </row>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <row gap="2" align="center">
        <spinner size="sm" tone="info"></spinner>
        <p style(mutedSm)>fetching…</p>
      </row>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Inline with status copy"
        lead="Pair a small spinner with a short wait message."
        code={spnExInline()}
      >
        <row gap="2" align="center" style(demoRow)>
          <spinner size="sm" tone="info"></spinner>
          <p style(mutedSm)>fetching…</p>
        </row>
      </doc-example>
      <doc-example title="Sizes" lead="Default is md." code={spnExSizes()}>
        <row gap="4" align="center" style(demoRow)>
          <spinner size="xs" tone="info"></spinner>
          <spinner size="sm" tone="info"></spinner>
          <spinner tone="info"></spinner>
          <spinner size="lg" tone="info"></spinner>
        </row>
      </doc-example>
      <doc-example
        title="Default tone"
        lead="tone defaults to info — a bare spinner is a finished wait ring."
        code={spnExDefault()}
      >
        <row gap="4" align="center" style(demoRow)>
          <spinner tone="info"></spinner>
          <spinner size="lg" tone="info"></spinner>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
