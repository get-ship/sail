<!--
  <doc-chip> — /components/chip
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style tickRow = { size({ w: full }).pad({ x: 1_5, t: 1 }) }
  style tick = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchPrimary = { bg(primary) }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDestructive = { bg(destructive) }
  style swatchAccent = { bg(accent) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style swatchInfo = { bg(info) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style demoRow = { size({ w: full }).pad({ y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-chip.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const CHIP_SIZES = ['sm', 'md', 'lg'] as const
  const variant = signal('solid')
  const sizeIdx = signal(1)
  const chipTone = signal('primary')
  const chipSelected = signal(false)
  const chipRemovable = signal(true)
  const kept = signal(true)
  const dropChip = (): void => kept.set(false)
  const restoreChip = (): void => kept.set(true)
  const pickChipTone = (t: string): void => chipTone.set(t)
  const chipSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), CHIP_SIZES.length - 1)
    return CHIP_SIZES[i] ?? 'md'
  })

  const entry = DETAILS.chip
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`variant="${variant()}"`]
    if (chipSize() !== 'md') a.push(`size="${chipSize()}"`)
    a.push(`tone="${chipTone()}"`)
    if (chipSelected()) a.push('selected')
    if (chipRemovable()) a.push('removable')
    return `<chip ${a.join(' ')}>${chipTone()}</chip>`
  })

  const chipExFilter = () =>
    `<chip variant="solid" tone="primary" selected>Design</chip>\n<chip variant="outline" tone="muted">Research</chip>`
  const chipExRemove = () =>
    `<chip variant="soft" tone="info" removable on-remove={onRemove}>typescript</chip>`
  const chipExDefault = () =>
    `<chip variant="solid" tone="primary">primary</chip>\n<chip variant="soft" tone="primary">primary</chip>\n<chip variant="outline" tone="primary">primary</chip>`

  return {
    variant,
    sizeIdx,
    chipTone,
    chipSelected,
    chipRemovable,
    chipSize,
    specimenCode,
    kept,
    dropChip,
    restoreChip,
    pickChipTone,
    heading,
    blurb,
    props,
    usageCode,
    chipExFilter,
    chipExRemove,
    chipExDefault,
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
    <column slot="specimen" gap="4" align="center">
      <if {kept()}>
        <chip
          variant={variant()}
          size={chipSize()}
          tone={chipTone()}
          selected={chipSelected() ? 'true' : null}
          removable={chipRemovable() ? 'true' : null}
          on-remove={dropChip}
        >
          {chipTone()}
        </chip>
      {else =>
        <button variant="ghost" size="sm" on:click={restoreChip}>bring it back</button>}</if>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Variant</label>
        <select size="sm" bind:value={variant} label="Variant">
          <option value="solid">solid</option>
          <option value="soft">soft</option>
          <option value="outline">outline</option>
          <option value="ghost">ghost</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="primary"
            selected={chipTone() === 'primary' ? 'true' : null}
            on:click={() => pickChipTone('primary')}
            style(swatch, swatchPrimary)
          ></div>
          <div
            title="success"
            selected={chipTone() === 'success' ? 'true' : null}
            on:click={() => pickChipTone('success')}
            style(swatch, swatchSuccess)
          ></div>
          <div
            title="warning"
            selected={chipTone() === 'warning' ? 'true' : null}
            on:click={() => pickChipTone('warning')}
            style(swatch, swatchWarning)
          ></div>
          <div
            title="destructive"
            selected={chipTone() === 'destructive' ? 'true' : null}
            on:click={() => pickChipTone('destructive')}
            style(swatch, swatchDestructive)
          ></div>
          <div
            title="accent"
            selected={chipTone() === 'accent' ? 'true' : null}
            on:click={() => pickChipTone('accent')}
            style(swatch, swatchAccent)
          ></div>
          <div
            title="muted"
            selected={chipTone() === 'muted' ? 'true' : null}
            on:click={() => pickChipTone('muted')}
            style(swatch, swatchMuted)
          ></div>
          <div
            title="info"
            selected={chipTone() === 'info' ? 'true' : null}
            on:click={() => pickChipTone('info')}
            style(swatch, swatchInfo)
          ></div>
        </row>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{chipSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size" ticks="sm,md,lg"></slider>
      </column>
      <row gap="5" align="start">
        <column gap="2" style({ grow(1).size({ min-w: 0 }) })>
          <label style(knobLabel)>Selected</label>
          <switch bind:checked={chipSelected} size="sm" label="Selected"></switch>
        </column>
        <column gap="2" style({ grow(1).size({ min-w: 0 }) })>
          <label style(knobLabel)>Removable</label>
          <switch bind:checked={chipRemovable} size="sm" label="Removable"></switch>
        </column>
      </row>
    </column>

    <column slot="usage-preview" gap="3" align="center">
      <chip variant="solid" tone="primary" selected>Design</chip>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Filter selection"
        lead="selected is the pressed state (ARIA-synced)."
        code={chipExFilter()}
      >
        <row gap="2" align="center" style(demoRow)>
          <chip variant="solid" tone="primary" selected>Design</chip>
          <chip variant="outline" tone="muted">Research</chip>
          <chip variant="soft" tone="info">Ops</chip>
        </row>
      </doc-example>

      <doc-example
        title="Removable tags"
        lead="removable shows the × and fires on-remove."
        code={chipExRemove()}
      >
        <row gap="2" align="center" style(demoRow)>
          <chip variant="soft" tone="info" removable>typescript</chip>
          <chip variant="solid" tone="success" removable size="sm">shipped</chip>
        </row>
      </doc-example>

      <doc-example
        title="Default tone"
        lead="tone defaults to primary — a bare chip is a finished control."
        code={chipExDefault()}
      >
        <row gap="2" align="center" style(demoRow)>
          <chip variant="solid" tone="primary">primary</chip>
          <chip variant="soft" tone="primary">primary</chip>
          <chip variant="outline" tone="primary">primary</chip>
          <chip variant="ghost" tone="primary">primary</chip>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
