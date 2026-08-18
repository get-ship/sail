<!--
  <doc-switch> — /components/switch
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style fieldShell = { size({ w: 100%, max-w: 64 }).pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchPrimary = { bg(primary) }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDestructive = { bg(destructive) }
  style swatchAccent = { bg(accent) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style swatchInfo = { bg(info) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style demoRow = { size({ w: 100% }).pad({ y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-switch.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const notify = signal(true)
  const switchTone = signal('primary')
  const switchDisabled = signal(false)
  const switchSize = signal('md')

  const pickTone = (t: string): void => switchTone.set(t)

  const entry = DETAILS.switch
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = ['bind:checked={notify}']
    if (switchSize() !== 'md') a.push(`size="${switchSize()}"`)
    a.push(`tone="${switchTone()}"`)
    if (switchDisabled()) a.push('disabled')
    return `<switch ${a.join(' ')} />`
  })

  const exSizes = () =>
    `<switch size="sm" checked />\n<switch checked />\n<switch size="lg" checked />`
  const exTones = () =>
    `<switch tone="success" checked />\n<switch tone="warning" checked />\n<switch tone="destructive" checked />`

  return {
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
        <label style(knobLabel)>Size</label>
        <select size="sm" bind:value={switchSize} label="Size">
          <option value="sm">sm</option>
          <option value="md">md</option>
          <option value="lg">lg</option>
        </select>
      </column>
      <column gap="2">
        <label><checkbox bind:checked={switchDisabled} size="sm" /> Disabled</label>
      </column>
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
