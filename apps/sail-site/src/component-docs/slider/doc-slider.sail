<!--
  <doc-slider> — /components/slider
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(64).maxW(full).pad(2).overflow(visible) }
  style controlW = { box: w(full) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style tickRow = { box: w(full).padT(1).padX(1_5) }
  style tick = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
</scope>

<controller>
import type { Scope } from './doc-slider.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const load = signal(60)
  const disabled = signal(false)

  const size = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SIZES.length - 1)
    return SIZES[i] ?? 'md'
  })

  const entry = DETAILS.slider
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = ['bind:value={volume}', 'min="0"', 'max="100"']
    if (size() !== 'md') a.push(`size="${size()}"`)
    if (disabled()) a.push('disabled')
    return `<slider ${a.join(' ')} />`
  })

  const exBind = () =>
    `<column gap="2">\n  <row justify="between"><span>Volume</span><span>{volume()}%</span></row>\n  <slider bind:value={volume} min="0" max="100" />\n</column>`
  const exSizes = () =>
    `<slider value="50" size="sm" />\n<slider value="50" />\n<slider value="50" size="lg" />`

  return {
    sizeIdx,
    size,
    load,
    disabled,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exBind,
    exSizes,
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
    <column slot="specimen" gap="3" align="stretch" style(specimen)>
      <row justify="between" align="center">
        <span style(sizeValue)>Value</span>
        <span style(sizeValue)>{load()}</span>
      </row>
      <if {disabled()}>
        <slider value={load()} min="0" max="100" step="1" size={size()} disabled style(controlW)></slider>
      {else =>
        <slider bind:value={load} min="0" max="100" step="1" size={size()} style(controlW)></slider>}</if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{size()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size" ticks="sm,md,lg"></slider>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Value</label>
          <span style(sizeValue)>{load()}</span>
        </row>
        <slider bind:value={load} min="0" max="100" size="sm" label="Value"></slider>
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={disabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" style(specimen)>
      <row justify="between"><span style(sizeValue)>Volume</span><span style(sizeValue)>{load()}%</span></row>
      <slider bind:value={load} min="0" max="100" style(controlW)></slider>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="bind:value"
        lead="Single thumb. Set min, max, and step as needed."
        code={exBind()}
      >
        <column gap="2" style(specimen)>
          <row justify="between"><span style(sizeValue)>Volume</span><span style(sizeValue)>{load()}%</span></row>
          <slider bind:value={load} min="0" max="100" style(controlW)></slider>
        </column>
      </doc-example>
      <doc-example
        title="Sizes"
        code={exSizes()}
      >
        <column gap="3" style(specimen)>
          <slider value="50" size="sm" style(controlW)></slider>
          <slider value="50" style(controlW)></slider>
          <slider value="50" size="lg" style(controlW)></slider>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/input">Input</a>
  </row>
</column>
