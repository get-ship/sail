<!--
  <doc-icon> — first-class docs host for /components/icon.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

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
  const iconMeta = derived(() => {
    const lab = iconLabeled() ? `label "${iconName()}"` : 'decorative'
    return `${iconName()} · ${iconSize()} · spin ${iconSpin() ? 'on' : 'off'} · ${lab}`
  })
  const iconLabelAttr = derived(() => (iconLabeled() ? iconName() : null))
  const entry = DETAILS.icon
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const iconExNamed = () =>
    `<icon name="check" size="sm"></icon>\n<button variant="solid" tone="primary">\n  <icon name="check" size="sm"></icon>\n  Save\n</button>`
  const iconExSizes = () =>
    `<icon name="sparkles" size="xs"></icon>\n<icon name="sparkles" size="sm"></icon>\n<icon name="sparkles"></icon>\n<icon name="sparkles" size="lg"></icon>\n<icon name="sparkles" size="xl"></icon>`
  const iconExSpin = () =>
    `<icon name="refresh-cw" spin label="Refreshing"></icon>`
  const iconExColor = () =>
    `<span style({ paint: fg(primary) })>\n  <icon name="heart"></icon>\n</span>`
  return {
    sizeIdx, iconSpin, iconName, iconLabeled, iconSize, iconMeta, iconLabelAttr,
    heading, blurb, props, usageCode, iconExNamed, iconExSizes, iconExSpin, iconExColor,
  }

})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style fullW = { box: w(full) }
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
  style tickRow = { box: w(full).padT(1) }
  style tick = {
    box: grow(1).basis(0).minW(0),
    paint: fg(muted-foreground),
    text: fontSize(xs).fontFamily(mono).textAlign(center),
  }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }

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
          <column gap="4" align="center" style(fullW)>
            <icon name={iconName()} size={iconSize()} spin={iconSpin() ? 'true' : null} label={iconLabelAttr()}></icon>
            <p style(monoXs)>{iconMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
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
            <p style(mutedXs)>Pack glyph via <code>name</code>. Use <code>src</code> only for same-origin SVG.</p>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{iconSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="4" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>xs</span><span style(tick)>sm</span><span style(tick)>md</span><span style(tick)>lg</span><span style(tick)>xl</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Spin</label>
            <row gap="2" align="center">
              <switch bind:checked={iconSpin} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{iconSpin() ? 'use:icon loop' : 'static'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Label</label>
            <row gap="2" align="center">
              <switch bind:checked={iconLabeled} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{iconLabeled() ? 'role=img' : 'decorative'}</span>
            </row>
          </column>
        </column>
      </row>
    </column>
    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Named pack glyphs</h3>
        <p style(exampleLead)>Prefer <code>name</code> for pack ids. Slot raw SVG only for one-offs.</p>
        <row gap="3" align="center">
          <icon name="check" size="sm"></icon>
          <icon name="search" size="sm"></icon>
          <icon name="settings" size="sm"></icon>
          <button variant="solid" tone="primary">
            <icon name="check" size="sm"></icon>
            Save
          </button>
        </row>
        <code-block code={iconExNamed()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)><code>xs</code>–<code>xl</code> map onto the theme icon-size scale.</p>
        <row gap="4" align="center">
          <icon name="sparkles" size="xs"></icon>
          <icon name="sparkles" size="sm"></icon>
          <icon name="sparkles"></icon>
          <icon name="sparkles" size="lg"></icon>
          <icon name="sparkles" size="xl"></icon>
        </row>
        <code-block code={iconExSizes()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Spin + label</h3>
        <p style(exampleLead)><code>spin</code> is owned by <code>use:icon</code>. Set <code>label</code> when meaningful.</p>
        <row gap="3" align="center">
          <icon name="refresh-cw" spin label="Refreshing"></icon>
        </row>
        <code-block code={iconExSpin()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>CurrentColor ink</h3>
        <p style(exampleLead)>No tone prop — ink is <code>currentColor</code> from the parent.</p>
        <row gap="3" align="center">
          <span style({ paint: fg(primary) })><icon name="heart"></icon></span>
          <span style({ paint: fg(destructive) })><icon name="heart"></icon></span>
        </row>
        <code-block code={iconExColor()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
