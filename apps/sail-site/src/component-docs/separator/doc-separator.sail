<!--
  <doc-separator> — /components/separator
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }) }
  style fullW = { box: size({ w: full }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style swatch = {
    box: size(7).flow({ shrink: 0 }),
    paint: radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs),
    when: { selected: { paint: border({ color: foreground }).elevation(sm) }, hover: { paint: filter({ opacity: 90 }) } },
}
  style swatchPrimary = { paint: fill({ color: primary }) }
  style swatchDestructive = { paint: fill({ color: destructive }) }
  style swatchMuted = { paint: fill({ color: muted }).border({ color: border }) }
  style swatchRest = { paint: fill({ color: background }).border({ color: border }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }) }
  style sectionLabel = { paint: ink({ color: muted-foreground }), text: type({ size: xs, tracking: wide, case: uppercase }) }
  style sepVRow = { box: size({ h: 10 }) }
  style sepStageCol = { box: size({ w: full, max-w: panel-max-w-sm }) }
</scope>

<controller>
import type { Scope } from './doc-separator.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const variant = signal('solid')
  const sepOrientation = signal('horizontal')
  const sepTone = signal('')
  const pickSepTone = (t: string): void => sepTone.set(t)
  const clearSepTone = (): void => sepTone.set('')

  const entry = DETAILS.separator
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (sepOrientation() !== 'horizontal') a.push(`orientation="${sepOrientation()}"`)
    if (variant() !== 'solid') a.push(`variant="${variant()}"`)
    if (sepTone()) a.push(`tone="${sepTone()}"`)
    return a.length ? `<separator ${a.join(' ')}></separator>` : `<separator></separator>`
  })

  const sepExBoth = () =>
    `<column gap="2">\n  <p>Section A</p>\n  <separator></separator>\n  <p>Section B</p>\n</column>`
  const sepExVariant = () =>
    `<separator variant="dashed"></separator>\n<separator variant="solid" tone="primary"></separator>`
  const sepExDefault = () =>
    `<separator></separator>\n<separator variant="dashed"></separator>`

  return {
    variant,
    sepOrientation,
    sepTone,
    specimenCode,
    pickSepTone,
    clearSepTone,
    heading,
    blurb,
    props,
    usageCode,
    sepExBoth,
    sepExVariant,
    sepExDefault,
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
    <column slot="specimen" gap="4" align="center" style(sepStageCol)>
      <if {sepOrientation() === 'vertical'}>
        <row gap="3" align="center" style(sepVRow)>
          <span style(mutedSm)>A</span>
          <if {sepTone()}>
            <separator orientation="vertical" variant={variant()} tone={sepTone()}></separator>
          {else =>
            <separator orientation="vertical" variant={variant()}></separator>}</if>
          <span style(mutedSm)>B</span>
        </row>
      {else =>
        <column gap="2" style(fullW)>
          <p style(mutedSm)>above</p>
          <if {sepTone()}>
            <separator orientation="horizontal" variant={variant()} tone={sepTone()}></separator>
          {else =>
            <separator orientation="horizontal" variant={variant()}></separator>}</if>
          <p style(mutedSm)>below</p>
        </column>}</if>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Orientation</label>
        <select size="sm" bind:value={sepOrientation} label="Orientation">
          <option value="horizontal">horizontal</option>
          <option value="vertical">vertical</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Variant</label>
        <select size="sm" bind:value={variant} label="Variant">
          <option value="solid">solid</option>
          <option value="dashed">dashed</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="default (theme)"
            selected={!sepTone() ? 'true' : null}
            on:click={clearSepTone}
            style(swatch, swatchRest)
          ></div>
          <div
            title="muted"
            selected={sepTone() === 'muted' ? 'true' : null}
            on:click={() => pickSepTone('muted')}
            style(swatch, swatchMuted)
          ></div>
          <div
            title="primary"
            selected={sepTone() === 'primary' ? 'true' : null}
            on:click={() => pickSepTone('primary')}
            style(swatch, swatchPrimary)
          ></div>
          <div
            title="destructive"
            selected={sepTone() === 'destructive' ? 'true' : null}
            on:click={() => pickSepTone('destructive')}
            style(swatch, swatchDestructive)
          ></div>
        </row>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(sepStageCol)>
      <column gap="2" style(fullW)>
        <p style(mutedSm)>Section A</p>
        <separator></separator>
        <p style(mutedSm)>Section B</p>
      </column>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Both orientations"
        lead="Horizontal rules sections; vertical sits between inline items."
        code={sepExBoth()}
      >
        <column gap="2" style(sepStageCol)>
          <p style(mutedSm)>Section A</p>
          <separator></separator>
          <p style(mutedSm)>Section B</p>
          <row gap="3" align="center" style(sepVRow)>
            <span style(mutedSm)>Left</span>
            <separator orientation="vertical"></separator>
            <span style(mutedSm)>Right</span>
          </row>
        </column>
      </doc-example>

      <doc-example
        title="Dashed and tone"
        lead='variant="dashed" draws a border stroke. Tone paints only when set.'
        code={sepExVariant()}
      >
        <column gap="3" style(sepStageCol)>
          <separator variant="dashed"></separator>
          <separator variant="solid" tone="primary"></separator>
          <separator variant="dashed" tone="destructive"></separator>
        </column>
      </doc-example>

      <doc-example
        title="Default (theme)"
        lead="No tone uses the theme’s default separator line."
        code={sepExDefault()}
      >
        <column gap="2" style(sepStageCol)>
          <separator></separator>
          <separator variant="dashed"></separator>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
