<!--
  <doc-chip> — first-class docs host for /components/chip.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

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
  const chipMeta = derived(() => {
    const t = chipTone()
    return `${variant()} · ${chipSize()} · ${t || 'rest (component tokens)'} · selected ${chipSelected() ? 'on' : 'off'} · removable ${chipRemovable() ? 'on' : 'off'}`
  })
  const entry = DETAILS.chip
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const chipExFilter = () =>
    `<chip variant="solid" tone="primary" selected>Design</chip>\n<chip variant="outline" tone="muted">Research</chip>`
  const chipExRemove = () =>
    `<chip variant="soft" tone="info" removable on-remove={onRemove}>typescript</chip>`
  const chipExRest = () =>
    `<chip variant="solid">rest tokens</chip>\n<!-- no tone → component.chip.* (themeable) -->`
  return {
    variant, sizeIdx, chipTone, chipSelected, chipRemovable, chipSize, chipMeta,
    kept, dropChip, restoreChip, pickChipTone, clearChipTone,
    heading, blurb, props, usageCode, chipExFilter, chipExRemove, chipExRest,
  }

})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageColMd = { box: w(panel-max-w-md).maxW(full) }
  style meterCol = { box: w(panel-max-w-sm) }
  style fullW = { box: w(full) }
  style grow1 = { box: grow(1) }
  style stageSplit = { box: w(full).minW(0).minH(64) }
  style stageCanvas = { box: grow(1).basis(0).minW(0).minH(64).pad(8) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style swatchRow = { box: w(full) }
  style tickRow = { box: w(full).padT(1) }
  style tick = {
    box: grow(1).basis(0).minW(0),
    paint: fg(muted-foreground),
    text: fontSize(xs).fontFamily(mono).textAlign(center),
  }
  style swatch = {
    box: w(7).h(7).shrink(0),
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
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style sepVRow = { box: h(10) }
  style sepStageCol = { box: w(full).maxW(panel-max-w-sm) }
  style skelStageCol = { box: w(full).maxW(panel-max-w-sm) }
  style skelBlock = { box: w(full).maxW(panel-max-w-sm) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }
</scope>

<column gap="4" style(page)>
  <sail-component-viewer
    name={heading()}
    blurb={blurb()}
    pack="Component primitives"
    variant-attr=""
    usage={usageCode()}
    variants={[]}
    rows={props()}
  >
    <column slot="stage" gap="4" align="stretch" style({ box: w(full).minW(0) })>
      <row nowrap gap="0" align="stretch" style(stageSplit)>
        <center style(stageCanvas)>
          <column gap="4" align="center">
            <if {kept()}>
              <if {chipTone()}>
                <chip variant={variant()} size={chipSize()} tone={chipTone()} selected={chipSelected() ? 'true' : null} removable={chipRemovable() ? 'true' : null} on-remove={dropChip}>{chipTone()}</chip>
              {else =>
                <chip variant={variant()} size={chipSize()} selected={chipSelected() ? 'true' : null} removable={chipRemovable() ? 'true' : null} on-remove={dropChip}>rest</chip>}</if>
            {else =>
              <button variant="ghost" size="sm" on:click={restoreChip}>bring it back</button>}</if>
            <p style(monoXs)>{chipMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
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
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="rest — component.chip tokens" placement="top">
                <div label="rest" on:click={clearChipTone} selected={!chipTone() ? 'true' : null} style(swatch, swatchRest)></div>
              </tooltip>
              <tooltip content="primary" placement="top">
                <div label="primary" on:click={() => pickChipTone('primary')} selected={chipTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
              </tooltip>
              <tooltip content="success" placement="top">
                <div label="success" on:click={() => pickChipTone('success')} selected={chipTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
              </tooltip>
              <tooltip content="warning" placement="top">
                <div label="warning" on:click={() => pickChipTone('warning')} selected={chipTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickChipTone('destructive')} selected={chipTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
              <tooltip content="accent" placement="top">
                <div label="accent" on:click={() => pickChipTone('accent')} selected={chipTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
              </tooltip>
              <tooltip content="muted" placement="top">
                <div label="muted" on:click={() => pickChipTone('muted')} selected={chipTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
              </tooltip>
              <tooltip content="info" placement="top">
                <div label="info" on:click={() => pickChipTone('info')} selected={chipTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
              </tooltip>
            </row>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{chipSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>sm</span><span style(tick)>md</span><span style(tick)>lg</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Selected</label>
            <row gap="2" align="center">
              <switch bind:checked={chipSelected} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{chipSelected() ? 'aria-pressed' : 'idle'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Removable</label>
            <row gap="2" align="center">
              <switch bind:checked={chipRemovable} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{chipRemovable() ? '× + Backspace' : 'no dismiss'}</span>
            </row>
          </column>
        </column>
      </row>
    </column>
    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Filter selection</h3>
        <p style(exampleLead)><code>selected</code> is the pressed state (ARIA-synced).</p>
        <row gap="2" align="center">
          <chip variant="solid" tone="primary" selected>Design</chip>
          <chip variant="outline" tone="muted">Research</chip>
          <chip variant="soft" tone="info">Ops</chip>
        </row>
        <code-block code={chipExFilter()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Removable tags</h3>
        <p style(exampleLead)><code>removable</code> shows the ×; fires <code>on-remove</code>.</p>
        <row gap="2" align="center">
          <chip variant="soft" tone="info" removable>typescript</chip>
          <chip variant="solid" tone="success" removable size="sm">shipped</chip>
        </row>
        <code-block code={chipExRemove()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>No <code>tone</code> → paint from <code>component.chip.*</code>.</p>
        <row gap="2" align="center">
          <chip variant="solid">solid rest</chip>
          <chip variant="soft">soft rest</chip>
          <chip variant="outline">outline rest</chip>
          <chip variant="ghost">ghost rest</chip>
        </row>
        <code-block code={chipExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
