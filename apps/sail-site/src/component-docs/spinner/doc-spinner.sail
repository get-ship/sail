<!--
  <doc-spinner> — /components/spinner
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style specimen = { box: size({ w: full }).space({ pad: 2 }).overflow({ all: visible }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style sizeValue = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style tickRow = { box: size({ w: full }).space({ pad-x: 1_5, pad-t: 1 }) }
  style tick = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style swatch = {
    box: size(7).flow({ shrink: 0 }),
    paint: radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs),
    when: { selected: { paint: border({ color: foreground }).elevation(sm) }, hover: { paint: filter({ opacity: 90 }) } },
}
  style swatchPrimary = { paint: fill({ color: primary }) }
  style swatchSuccess = { paint: fill({ color: success }) }
  style swatchWarning = { paint: fill({ color: warning }) }
  style swatchDestructive = { paint: fill({ color: destructive }) }
  style swatchAccent = { paint: fill({ color: accent }) }
  style swatchInfo = { paint: fill({ color: info }) }
  style swatchRest = { paint: fill({ color: background }).border({ color: border }) }
  style centerBox = { box: space({ pad: 4 }), paint: border({ color: border-subtle, width: 1 }).radius(md) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
  style demoRow = { box: size({ w: full }).space({ pad-y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-spinner.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SPN_SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const spinnerTone = signal('primary')
  const spinnerCentered = signal(false)

  const pickSpinnerTone = (t: string): void => spinnerTone.set(t)
  const clearSpinnerTone = (): void => spinnerTone.set('')

  const spinnerSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SPN_SIZES.length - 1)
    return SPN_SIZES[i] ?? 'md'
  })

  const entry = DETAILS.spinner
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (spinnerSize() !== 'md') a.push(`size="${spinnerSize()}"`)
    if (spinnerTone()) a.push(`tone="${spinnerTone()}"`)
    if (spinnerCentered()) a.push('centered')
    return a.length ? `<spinner ${a.join(' ')}></spinner>` : `<spinner></spinner>`
  })

  const spnExInline = () =>
    `<row gap="2" align="center">\n  <spinner size="sm" tone="primary"></spinner>\n  <p>fetching…</p>\n</row>`
  const spnExSizes = () =>
    `<spinner size="xs" tone="primary"></spinner>\n<spinner size="sm" tone="primary"></spinner>\n<spinner tone="primary"></spinner>\n<spinner size="lg" tone="primary"></spinner>`
  const spnExDefault = () => `<spinner></spinner>`
  const spnExCentered = () => `<spinner centered tone="primary"></spinner>`

  return {
    sizeIdx,
    spinnerTone,
    spinnerCentered,
    spinnerSize,
    pickSpinnerTone,
    clearSpinnerTone,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    spnExInline,
    spnExSizes,
    spnExDefault,
    spnExCentered,
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
      <if {spinnerTone()}>
        <spinner
          size={spinnerSize()}
          tone={spinnerTone()}
          centered={spinnerCentered() ? 'true' : null}
        ></spinner>
      {else =>
        <spinner size={spinnerSize()} centered={spinnerCentered() ? 'true' : null}></spinner>}</if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{spinnerSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="3" step="1" size="sm" label="Size" ticks="xs,sm,md,lg"></slider>
      </column>
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="default"
            selected={!spinnerTone() ? 'true' : null}
            on:click={clearSpinnerTone}
            style(swatch, swatchRest)
          ></div>
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
            title="destructive"
            selected={spinnerTone() === 'destructive' ? 'true' : null}
            on:click={() => pickSpinnerTone('destructive')}
            style(swatch, swatchDestructive)
          ></div>
          <div
            title="accent"
            selected={spinnerTone() === 'accent' ? 'true' : null}
            on:click={() => pickSpinnerTone('accent')}
            style(swatch, swatchAccent)
          ></div>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Centered</label>
        <row gap="2" align="center">
          <switch bind:checked={spinnerCentered} size="sm" tone="primary"></switch>
          <span style(sizeValue)>{spinnerCentered() ? 'block + mx auto' : 'inline'}</span>
        </row>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <row gap="2" align="center">
        <spinner size="sm" tone="primary"></spinner>
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
          <spinner size="sm" tone="primary"></spinner>
          <p style(mutedSm)>fetching…</p>
        </row>
      </doc-example>
      <doc-example title="Sizes" lead="Default is md." code={spnExSizes()}>
        <row gap="4" align="center" style(demoRow)>
          <spinner size="xs" tone="primary"></spinner>
          <spinner size="sm" tone="primary"></spinner>
          <spinner tone="primary"></spinner>
          <spinner size="lg" tone="primary"></spinner>
        </row>
      </doc-example>
      <doc-example
        title="Default theme colors"
        lead="Omit tone for default theme colors."
        code={spnExDefault()}
      >
        <row gap="4" align="center" style(demoRow)>
          <spinner></spinner>
          <spinner size="lg"></spinner>
        </row>
      </doc-example>
      <doc-example
        title="Centered"
        lead="centered makes the host a block with auto horizontal margins."
        code={spnExCentered()}
      >
        <column gap="2" style(centerBox)>
          <spinner centered tone="primary"></spinner>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
