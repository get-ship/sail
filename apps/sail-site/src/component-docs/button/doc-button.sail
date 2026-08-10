<!--
  <doc-button> — first-class docs host for /components/button.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-button.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const BTN_SIZES = ['xs', 'sm', 'md', 'lg', 'icon', 'iconSm', 'iconLg'] as const
  const variant = signal('solid')
  const sizeIdx = signal(2)
  const btnTone = signal('primary')
  const presses = signal(0)
  const press = (): void => presses.set((n) => n + 1)
  const pickTone = (t: string): void => btnTone.set(t)

  const btnSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), BTN_SIZES.length - 1)
    return BTN_SIZES[i] ?? 'md'
  })
  const isIconSize = derived(() => {
    const s = btnSize()
    return s === 'icon' || s === 'iconSm' || s === 'iconLg'
  })
  const btnMeta = derived(() => `${variant()} · ${btnSize()} · ${btnTone()} · pressed ${presses()}`)

  const entry = DETAILS.button
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const btnExWithIcon = () =>
    `<button variant="solid" tone="primary">\n  <icon name="check" size="sm"></icon>\n  Save\n</button>`
  const btnExIconOnly = () =>
    `<button variant="solid" tone="primary" size="icon" label="Settings">\n  <icon name="settings"></icon>\n</button>\n<button variant="ghost" size="iconSm" label="Search">\n  <icon name="search" size="sm"></icon>\n</button>`
  const btnExSizes = () =>
    `<button variant="solid" tone="primary" size="xs">xs</button>\n<button variant="solid" tone="primary" size="lg">lg</button>`

  return {
    variant,
    sizeIdx,
    btnTone,
    btnSize,
    isIconSize,
    btnMeta,
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

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
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
  style pressBtn = { animate: press({ scale: 'press' }) }
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
          <column gap="4" align="center">
            <if {isIconSize()}>
              <button
                variant={variant()}
                size={btnSize()}
                tone={btnTone()}
                on:click={press}
                label="Settings"
                style(pressBtn)
              >
                <icon name="settings"></icon>
              </button>
            {else =>
              <button
                variant={variant()}
                size={btnSize()}
                tone={btnTone()}
                on:click={press}
                style(pressBtn)
              >
                <icon name="check" size="sm"></icon>
                Save changes
              </button>}</if>
            <p style(monoXs)>{btnMeta()}</p>
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
              <tooltip content="primary" placement="top">
                <div label="primary" on:click={() => pickTone('primary')} selected={btnTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
              </tooltip>
              <tooltip content="success" placement="top">
                <div label="success" on:click={() => pickTone('success')} selected={btnTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
              </tooltip>
              <tooltip content="warning" placement="top">
                <div label="warning" on:click={() => pickTone('warning')} selected={btnTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickTone('destructive')} selected={btnTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
              <tooltip content="accent" placement="top">
                <div label="accent" on:click={() => pickTone('accent')} selected={btnTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
              </tooltip>
              <tooltip content="muted" placement="top">
                <div label="muted" on:click={() => pickTone('muted')} selected={btnTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
              </tooltip>
              <tooltip content="info" placement="top">
                <div label="info" on:click={() => pickTone('info')} selected={btnTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
              </tooltip>
            </row>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{btnSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="6" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>xs</span>
              <span style(tick)>sm</span>
              <span style(tick)>md</span>
              <span style(tick)>lg</span>
              <span style(tick)>icon</span>
              <span style(tick)>iSm</span>
              <span style(tick)>iLg</span>
            </row>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>With icons</h3>
        <p style(exampleLead)>
          Children compose: put <code>&lt;icon&gt;</code> before or after the label.
          The button is inline-flex with gap — no left/right section props.
        </p>
        <row gap="2" align="center">
          <button variant="solid" tone="primary">
            <icon name="check" size="sm"></icon>
            Save
          </button>
          <button variant="outline">
            Export
            <icon name="download" size="sm"></icon>
          </button>
        </row>
        <code-block code={btnExWithIcon()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Icon-only</h3>
        <p style(exampleLead)>
          Use <code>size="icon"</code>, <code>iconSm</code>, or <code>iconLg</code> for a square hit target.
          Always set <code>label</code> for accessible name when there is no text.
        </p>
        <row gap="2" align="center">
          <button variant="solid" tone="primary" size="icon" label="Settings">
            <icon name="settings"></icon>
          </button>
          <button variant="ghost" size="iconSm" label="Search">
            <icon name="search" size="sm"></icon>
          </button>
          <button variant="outline" size="iconLg" label="Add">
            <icon name="plus" size="lg"></icon>
          </button>
        </row>
        <code-block code={btnExIconOnly()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Sizes</h3>
        <p style(exampleLead)>
          Text controls: <code>xs</code>–<code>lg</code>. Defaults to <code>md</code>.
          Drive the live stage above for the full matrix; these are the named steps.
        </p>
        <row gap="2" align="center">
          <button variant="solid" tone="primary" size="xs">xs</button>
          <button variant="solid" tone="primary" size="sm">sm</button>
          <button variant="solid" tone="primary" size="md">md</button>
          <button variant="solid" tone="primary" size="lg">lg</button>
        </row>
        <code-block code={btnExSizes()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
