<!--
  <doc-separator> — /components/separator
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }) }
  style fullW = { size({ w: 100% }) }
  style mutedSm = { fg(muted-foreground).type({ size: sm }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchPrimary = { bg(primary) }
  style swatchDanger = { bg(danger) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style sepVRow = { size({ h: 10 }) }
  style sepStageCol = { size({ w: 100%, max-w: overlay-max-w-sm, min-h: 11 }) }
</scope>

<controller>
import type { Scope } from './doc-separator.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const variant = signal('solid')
  const sepOrientation = signal('horizontal')
  const sepTone = signal('muted')
  const pickSepTone = (t: string): void => sepTone.set(t)

  const entry = DETAILS.separator
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (sepOrientation() !== 'horizontal') a.push(`orientation="${sepOrientation()}"`)
    if (variant() !== 'solid') a.push(`variant="${variant()}"`)
    a.push(`tone="${sepTone()}"`)
    return `<separator ${a.join(' ')}></separator>`
  })

  const sepExBoth = () =>
    `<column gap="2">\n  <p>Section A</p>\n  <separator></separator>\n  <p>Section B</p>\n</column>`
  const sepExVariant = () =>
    `<separator variant="dashed"></separator>\n<separator variant="solid" tone="primary"></separator>`
  const sepExDefault = () =>
    `<separator tone="muted"></separator>\n<separator variant="dashed" tone="muted"></separator>`

  return {
    variant,
    sepOrientation,
    sepTone,
    specimenCode,
    pickSepTone,
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
          <separator orientation="vertical" variant={variant()} tone={sepTone()}></separator>
          <span style(mutedSm)>B</span>
        </row>
      {else =>
        <column gap="2" style(fullW)>
          <p style(mutedSm)>above</p>
          <separator orientation="horizontal" variant={variant()} tone={sepTone()}></separator>
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
            title="danger"
            selected={sepTone() === 'danger' ? 'true' : null}
            on:click={() => pickSepTone('danger')}
            style(swatch, swatchDanger)
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
        lead='variant="dashed" draws a border stroke. tone defaults to muted.'
        code={sepExVariant()}
      >
        <column gap="3" style(sepStageCol)>
          <separator variant="dashed"></separator>
          <separator variant="solid" tone="primary"></separator>
          <separator variant="dashed" tone="danger"></separator>
        </column>
      </doc-example>

      <doc-example
        title="Default tone"
        lead="tone defaults to muted — a finished divider, not an empty swatch."
        code={sepExDefault()}
      >
        <column gap="2" style(sepStageCol)>
          <separator tone="muted"></separator>
          <separator variant="dashed" tone="muted"></separator>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
