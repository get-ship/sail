<!--
  <doc-button> — first-class docs host for /components/button.
  Layout SSOT: sail-component-viewer slots specimen | knobs (row, not stacked).
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }) }
  style monoXs = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  /**
   * Discrete steps: padX ≈ half thumb so ends sit under the thumb center;
   * justify-between places labels at 0%…100% (not column midpoints).
   */
  style tickRow = { size({ w: full }).pad({ x: 1_5, t: 1 }) }
  style tick = { fg(muted-foreground).type({ family: mono, size: xs }) }
  /** Tone dots — plain divs (never <button>). size() sets w+h together. */
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchPrimary = { bg(primary) }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDestructive = { bg(destructive) }
  style swatchAccent = { bg(accent) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style swatchInfo = { bg(info) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }) }
  style exampleSection = { size({ w: full }).pad({ y: 4 }) }
  style exampleH = { type({ size: md, weight: semibold }) }
  style exampleLead = { fg(muted-foreground).type({ size: sm }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style demoRow = { size({ w: full }).pad({ y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-button.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const BTN_SIZES = ['xs', 'sm', 'md', 'lg', 'icon', 'iconSm', 'iconLg'] as const

  const variant = signal('solid')
  const sizeIdx = signal(2)
  const btnTone = signal('primary')
  const disabled = signal(false)
  const withIcon = signal(false)
  const presses = signal(0)

  const press = (): void => {
    if (disabled()) return
    presses.set((n) => n + 1)
  }
  const pickTone = (t: string): void => btnTone.set(t)

  const btnSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), BTN_SIZES.length - 1)
    return BTN_SIZES[i] ?? 'md'
  })
  const isIconSize = derived(() => {
    const s = btnSize()
    return s === 'icon' || s === 'iconSm' || s === 'iconLg'
  })
  /** Live markup for the stage specimen — copy/paste as-is. */
  const specimenCode = derived(() => {
    const attrs = [`variant="${variant()}"`, `tone="${btnTone()}"`, `size="${btnSize()}"`]
    if (disabled()) attrs.push('disabled')
    const open = `<button ${attrs.join(' ')}`
    if (isIconSize()) {
      return `${open} label="Settings">\n  <icon name="settings"></icon>\n</button>`
    }
    if (withIcon()) {
      return `${open}>\n  <icon name="check" size="sm"></icon>\n  Save changes\n</button>`
    }
    return `${open}>\n  Save changes\n</button>`
  })
  const entry = DETAILS.button
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () =>
    entry.props.filter((r) =>
        r.name === 'variant' ||
        r.name === 'tone' ||
        r.name === 'size' ||
        r.name === 'disabled' ||
        r.name === 'label')
  const usageCode = () => entry.usage

  const btnExWithIcon = () =>
    `<button variant="solid" tone="primary">\n  <icon name="check" size="sm"></icon>\n  Save\n</button>\n<button variant="outline" tone="primary">\n  Export\n  <icon name="download" size="sm"></icon>\n</button>`
  const btnExIconOnly = () =>
    `<button variant="solid" tone="primary" size="icon" label="Settings">\n  <icon name="settings"></icon>\n</button>\n<button variant="ghost" tone="primary" size="iconSm" label="Search">\n  <icon name="search" size="sm"></icon>\n</button>\n<button variant="outline" tone="primary" size="iconLg" label="Add">\n  <icon name="plus" size="lg"></icon>\n</button>`
  const btnExSizes = () =>
    `<button variant="solid" tone="primary" size="xs">xs</button>\n<button variant="solid" tone="primary" size="sm">sm</button>\n<button variant="solid" tone="primary" size="md">md</button>\n<button variant="solid" tone="primary" size="lg">lg</button>`

  return {
    variant,
    sizeIdx,
    btnTone,
    disabled,
    btnSize,
    isIconSize,
    withIcon,
    presses,
    specimenCode,
    press,
    pickTone,
    heading,
    blurb,
    props,
    usageCode,
    btnExWithIcon,
    btnExIconOnly,
    btnExSizes,
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
      <if {isIconSize()}>
        <button
          variant={variant()}
          size={btnSize()}
          tone={btnTone()}
          disabled={disabled()}
          on:click={press}
          label="Settings"
        >
          <icon name="settings"></icon>
        </button>
      {else =>
        <if {withIcon()}>
          <button
            variant={variant()}
            size={btnSize()}
            tone={btnTone()}
            disabled={disabled()}
            on:click={press}
          >
            <icon name="check" size="sm"></icon>
            Save changes
          </button>
        {else =>
          <button
            variant={variant()}
            size={btnSize()}
            tone={btnTone()}
            disabled={disabled()}
            on:click={press}
          >
            Save changes
          </button>}</if>}</if>
    </column>

    <column slot="usage-preview" gap="3" align="center">
      <button variant="solid" tone="primary" on:click={press}>
        {presses() > 0 ? 'Saved' : 'Save changes'}
      </button>
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
            selected={btnTone() === 'primary' ? 'true' : null}
            on:click={() => pickTone('primary')}
            style(swatch, swatchPrimary)
          ></div>
          <div
            title="success"
            selected={btnTone() === 'success' ? 'true' : null}
            on:click={() => pickTone('success')}
            style(swatch, swatchSuccess)
          ></div>
          <div
            title="warning"
            selected={btnTone() === 'warning' ? 'true' : null}
            on:click={() => pickTone('warning')}
            style(swatch, swatchWarning)
          ></div>
          <div
            title="destructive"
            selected={btnTone() === 'destructive' ? 'true' : null}
            on:click={() => pickTone('destructive')}
            style(swatch, swatchDestructive)
          ></div>
          <div
            title="accent"
            selected={btnTone() === 'accent' ? 'true' : null}
            on:click={() => pickTone('accent')}
            style(swatch, swatchAccent)
          ></div>
          <div
            title="muted"
            selected={btnTone() === 'muted' ? 'true' : null}
            on:click={() => pickTone('muted')}
            style(swatch, swatchMuted)
          ></div>
          <div
            title="info"
            selected={btnTone() === 'info' ? 'true' : null}
            on:click={() => pickTone('info')}
            style(swatch, swatchInfo)
          ></div>
        </row>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{btnSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="6" step="1" size="sm" label="Size" ticks="xs,sm,md,lg,icon,iSm,iLg"></slider>
      </column>
      <row gap="5" align="start">
        <column gap="2" style({ grow(1).size({ min-w: 0 }) })>
          <label style(knobLabel)>Icon</label>
          <if {isIconSize()}>
            <p style(sizeValue)>Required for icon sizes</p>
          {else =>
            <switch bind:checked={withIcon} size="sm" label="Show icon"></switch>}</if>
        </column>
        <column gap="2" style({ grow(1).size({ min-w: 0 }) })>
          <label style(knobLabel)>Disabled</label>
          <switch bind:checked={disabled} size="sm" label="Disabled"></switch>
        </column>
      </row>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="With icons"
        lead="Put an icon before or after the label."
        code={btnExWithIcon()}
      >
        <row gap="2" align="center" style(demoRow)>
          <button variant="solid" tone="primary">
            <icon name="check" size="sm"></icon>
            Save
          </button>
          <button variant="outline" tone="primary">
            Export
            <icon name="download" size="sm"></icon>
          </button>
        </row>
      </doc-example>

      <doc-example
        title="Icon-only"
        lead='Square targets with size="icon" (or iconSm / iconLg). Set label when there is no visible text.'
        code={btnExIconOnly()}
      >
        <row gap="2" align="center" style(demoRow)>
          <button variant="solid" tone="primary" size="icon" label="Settings">
            <icon name="settings"></icon>
          </button>
          <button variant="ghost" tone="primary" size="iconSm" label="Search">
            <icon name="search" size="sm"></icon>
          </button>
          <button variant="outline" tone="primary" size="iconLg" label="Add">
            <icon name="plus" size="lg"></icon>
          </button>
        </row>
      </doc-example>

      <doc-example
        title="Sizes"
        lead="From compact xs to roomy lg. Default is md."
        code={btnExSizes()}
      >
        <row gap="2" align="center" style(demoRow)>
          <button variant="solid" tone="primary" size="xs">xs</button>
          <button variant="solid" tone="primary" size="sm">sm</button>
          <button variant="solid" tone="primary" size="md">md</button>
          <button variant="solid" tone="primary" size="lg">lg</button>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
