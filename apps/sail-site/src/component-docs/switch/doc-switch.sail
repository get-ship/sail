<!--
  <doc-switch> — /components/switch
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style fieldShell = { box: w(full).maxW(64).pad(2).overflow(visible) }
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
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
  style demoRow = { box: w(full).padY(2) }
</scope>

<controller>
import type { Scope } from './doc-switch.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const notify = signal(true)
  const switchTone = signal('primary')
  const switchDisabled = signal(false)

  const pickTone = (t: string): void => switchTone.set(t)
  const switchSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SIZES.length - 1)
    return SIZES[i] ?? 'md'
  })

  const entry = DETAILS.switch
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = ['bind:checked={on}']
    if (switchSize() !== 'md') a.push(`size="${switchSize()}"`)
    if (switchTone()) a.push(`tone="${switchTone()}"`)
    if (switchDisabled()) a.push('disabled')
    return `<switch ${a.join(' ')} />`
  })

  const exSizes = () =>
    `<switch size="sm" checked />\n<switch checked />\n<switch size="lg" checked />`
  const exTones = () =>
    `<switch tone="success" checked />\n<switch tone="warning" checked />\n<switch tone="destructive" checked />`

  return {
    sizeIdx,
    notify,
    switchTone,
    switchDisabled,
    switchSize,
    pickTone,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exSizes,
    exTones,
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
    <column slot="specimen" gap="3" align="center" style(fieldShell)>
      <row gap="3" align="center">
        <switch
          bind:checked={notify}
          size={switchSize()}
          tone={switchTone()}
          disabled={switchDisabled()}
        ></switch>
        <span style(sizeValue)>{notify() ? 'On' : 'Off'}</span>
      </row>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div title="primary" selected={switchTone() === 'primary' ? 'true' : null} on:click={() => pickTone('primary')} style(swatch, swatchPrimary)></div>
          <div title="success" selected={switchTone() === 'success' ? 'true' : null} on:click={() => pickTone('success')} style(swatch, swatchSuccess)></div>
          <div title="warning" selected={switchTone() === 'warning' ? 'true' : null} on:click={() => pickTone('warning')} style(swatch, swatchWarning)></div>
          <div title="destructive" selected={switchTone() === 'destructive' ? 'true' : null} on:click={() => pickTone('destructive')} style(swatch, swatchDestructive)></div>
          <div title="accent" selected={switchTone() === 'accent' ? 'true' : null} on:click={() => pickTone('accent')} style(swatch, swatchAccent)></div>
          <div title="muted" selected={switchTone() === 'muted' ? 'true' : null} on:click={() => pickTone('muted')} style(swatch, swatchMuted)></div>
          <div title="info" selected={switchTone() === 'info' ? 'true' : null} on:click={() => pickTone('info')} style(swatch, swatchInfo)></div>
        </row>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{switchSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size" ticks="sm,md,lg"></slider>
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={notify} size="sm" /> On</label>
        <label><checkbox bind:checked={switchDisabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(fieldShell)>
      <row gap="3" align="center">
        <switch bind:checked={notify}></switch>
        <span style(sizeValue)>{notify() ? 'On' : 'Off'}</span>
      </row>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Sizes"
        lead="Default is md."
        code={exSizes()}
      >
        <row gap="4" align="center" style(demoRow)>
          <switch size="sm" checked></switch>
          <switch checked></switch>
          <switch size="lg" checked></switch>
        </row>
      </doc-example>
      <doc-example
        title="Tone"
        lead="Tone paints the on track. Off always uses the switch surface."
        code={exTones()}
      >
        <row gap="3" align="center" style(demoRow)>
          <switch tone="success" checked></switch>
          <switch tone="warning" checked></switch>
          <switch tone="destructive" checked></switch>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/checkbox">Checkbox</a>
  </row>
</column>
