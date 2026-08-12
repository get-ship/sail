<!--
  <doc-chip> — /components/chip
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style tickRow = { box: w(full).padT(1).padX(1_5) }
  style tick = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style swatch = {
    box: size(7).shrink(0),
    paint: radius(full).cursor(pointer).borderWidth(2).border(transparent).shadow(xs),
    when: {
      selected: { paint: border(foreground).shadow(sm) },
      hover: { paint: opacity(90) },
    },
  }
  style swatchPrimary = { paint: bg(primary) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted).border(border) }
  style swatchInfo = { paint: bg(info) }
  style swatchRest = { paint: bg(background).border(border) }
  style exampleBlock = { box: w(full).padT(2).padB(4) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style demoRow = { box: w(full).padY(2) }
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
  const clearChipTone = (): void => chipTone.set('')
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
    if (chipTone()) a.push(`tone="${chipTone()}"`)
    if (chipSelected()) a.push('selected')
    if (chipRemovable()) a.push('removable')
    const label = chipTone() || 'default'
    return `<chip ${a.join(' ')}>${label}</chip>`
  })

  const chipExFilter = () =>
    `<chip variant="solid" tone="primary" selected>Design</chip>\n<chip variant="outline" tone="muted">Research</chip>`
  const chipExRemove = () =>
    `<chip variant="soft" tone="info" removable on-remove={onRemove}>typescript</chip>`
  const chipExDefault = () =>
    `<chip variant="solid">default</chip>\n<chip variant="soft">default</chip>\n<chip variant="outline">default</chip>`

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
    clearChipTone,
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
        <if {chipTone()}>
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
          <chip
            variant={variant()}
            size={chipSize()}
            selected={chipSelected() ? 'true' : null}
            removable={chipRemovable() ? 'true' : null}
            on-remove={dropChip}
          >
            default
          </chip>}</if>
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
            title="default (theme)"
            selected={!chipTone() ? 'true' : null}
            on:click={clearChipTone}
            style(swatch, swatchRest)
          ></div>
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
        <column gap="2" style({ box: grow(1).minW(0) })>
          <label style(knobLabel)>Selected</label>
          <switch bind:checked={chipSelected} size="sm" label="Selected"></switch>
        </column>
        <column gap="2" style({ box: grow(1).minW(0) })>
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
        title="Default (theme)"
        lead="No tone uses the theme’s default chip colors."
        code={chipExDefault()}
      >
        <row gap="2" align="center" style(demoRow)>
          <chip variant="solid">default</chip>
          <chip variant="soft">default</chip>
          <chip variant="outline">default</chip>
          <chip variant="ghost">default</chip>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
