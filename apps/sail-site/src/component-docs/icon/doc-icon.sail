<!--
  <doc-icon> — /components/icon
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }) }
  style mutedXs = { fg(muted-foreground).type({ size: xs }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style tickRow = { size({ w: full }).pad({ x: 1_5, t: 1 }) }
  style tick = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style demoRow = { size({ w: full }).pad({ y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-icon.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const ICON_SIZES = ['xs', 'sm', 'md', 'lg', 'xl'] as const
  const sizeIdx = signal(2)
  const iconSpin = signal(false)
  const iconName = signal('sparkles')
  const iconLabeled = signal(false)
  const iconSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), ICON_SIZES.length - 1)
    return ICON_SIZES[i] ?? 'md'
  })
  const iconLabelAttr = derived(() => (iconLabeled() ? iconName() : null))

  const entry = DETAILS.icon
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`name="${iconName()}"`]
    if (iconSize() !== 'md') a.push(`size="${iconSize()}"`)
    if (iconSpin()) a.push('spin')
    if (iconLabeled()) a.push(`label="${iconName()}"`)
    return `<icon ${a.join(' ')}></icon>`
  })

  const iconExNamed = () =>
    `<icon name="check" size="sm"></icon>\n<button variant="solid" tone="primary">\n  <icon name="check" size="sm"></icon>\n  Save\n</button>`
  const iconExSizes = () =>
    `<icon name="sparkles" size="xs"></icon>\n<icon name="sparkles" size="sm"></icon>\n<icon name="sparkles"></icon>\n<icon name="sparkles" size="lg"></icon>\n<icon name="sparkles" size="xl"></icon>`
  const iconExSpin = () =>
    `<icon name="refresh-cw" spin label="Refreshing"></icon>`
  const iconExColor = () =>
    `<span style({ fg(primary) })>\n  <icon name="heart"></icon>\n</span>`

  return {
    sizeIdx,
    iconSpin,
    iconName,
    iconLabeled,
    iconSize,
    iconLabelAttr,
    specimenCode,
    heading,
    blurb,
    props,
    usageCode,
    iconExNamed,
    iconExSizes,
    iconExSpin,
    iconExColor,
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
      <icon
        name={iconName()}
        size={iconSize()}
        spin={iconSpin() ? 'true' : null}
        label={iconLabelAttr()}
      ></icon>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Name</label>
        <select size="sm" bind:value={iconName} label="Name">
          <option value="sparkles">sparkles</option>
          <option value="check">check</option>
          <option value="settings">settings</option>
          <option value="search">search</option>
          <option value="heart">heart</option>
          <option value="refresh-cw">refresh-cw</option>
        </select>
        <p style(mutedXs)>Pack glyph via name. Use src only for same-origin SVG.</p>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{iconSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="4" step="1" size="sm" label="Size" ticks="xs,sm,md,lg,xl"></slider>
      </column>
      <row gap="5" align="start">
        <column gap="2" style({ grow(1).size({ min-w: 0 }) })>
          <label style(knobLabel)>Spin</label>
          <switch bind:checked={iconSpin} size="sm" label="Spin"></switch>
        </column>
        <column gap="2" style({ grow(1).size({ min-w: 0 }) })>
          <label style(knobLabel)>Label</label>
          <switch bind:checked={iconLabeled} size="sm" label="Accessible label"></switch>
        </column>
      </row>
    </column>

    <column slot="usage-preview" gap="3" align="center">
      <icon name="sparkles"></icon>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Named pack glyphs"
        lead="Prefer name for pack ids. Slot raw SVG only for one-offs."
        code={iconExNamed()}
      >
        <row gap="3" align="center" style(demoRow)>
          <icon name="check" size="sm"></icon>
          <icon name="search" size="sm"></icon>
          <icon name="settings" size="sm"></icon>
          <button variant="solid" tone="primary">
            <icon name="check" size="sm"></icon>
            Save
          </button>
        </row>
      </doc-example>

      <doc-example
        title="Size ladder"
        lead="xs–xl map onto the theme icon-size scale."
        code={iconExSizes()}
      >
        <row gap="4" align="center" style(demoRow)>
          <icon name="sparkles" size="xs"></icon>
          <icon name="sparkles" size="sm"></icon>
          <icon name="sparkles"></icon>
          <icon name="sparkles" size="lg"></icon>
          <icon name="sparkles" size="xl"></icon>
        </row>
      </doc-example>

      <doc-example
        title="Spin + label"
        lead="spin animates the glyph. Set label when the icon is meaningful alone."
        code={iconExSpin()}
      >
        <row gap="3" align="center" style(demoRow)>
          <icon name="refresh-cw" spin label="Refreshing"></icon>
        </row>
      </doc-example>

      <doc-example
        title="CurrentColor ink"
        lead="No tone prop — ink is currentColor from the parent."
        code={iconExColor()}
      >
        <row gap="3" align="center" style(demoRow)>
          <span style({ fg(primary) })><icon name="heart"></icon></span>
          <span style({ fg(destructive) })><icon name="heart"></icon></span>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
