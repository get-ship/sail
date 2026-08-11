<!--
  <doc-switch> — first-class docs host for /components/switch.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-switch.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SWITCH_SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const notify = signal(true)
  const switchTone = signal('primary')
  const switchDisabled = signal(false)
  const pickSwitchTone = (t: string): void => switchTone.set(t)
  const clearSwitchTone = (): void => switchTone.set('')
  const switchSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SWITCH_SIZES.length - 1)
    return SWITCH_SIZES[i] ?? 'md'
  })
  const switchMeta = derived(() => {
    const t = switchTone()
    const on = notify() ? 'on' : 'off'
    const d = switchDisabled() ? 'disabled' : 'enabled'
    return `${switchSize()} · ${t || 'no tone (checked→primary)'} · ${on} · ${d} · thumb switch.thumb`
  })
  const entry = DETAILS.switch
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const swExBind = () =>
    `// notify = signal(true)\n<row gap="3" align="center">\n  <switch bind:checked={notify}></switch>\n  <p>{notify() ? 'on' : 'off'}</p>\n</row>`
  const swExSizes = () =>
    `<switch size="sm" checked></switch>\n<switch checked></switch>\n<switch size="lg" checked></switch>`
  const swExTones = () =>
    `<switch tone="success" checked></switch>\n<switch tone="warning" checked></switch>\n<switch tone="destructive" checked></switch>\n<!-- tone paints checked track only; thumb stays switch.thumb -->`
  const swExRest = () =>
    `<switch></switch>\n<switch checked></switch>\n<!-- off → component.switch.track + border; checked (no tone) → primary; thumb → switch.thumb -->`
  return {
    sizeIdx, notify, switchTone, switchDisabled, switchSize, switchMeta,
    pickSwitchTone, clearSwitchTone,
    heading, blurb, props, usageCode,
    swExBind, swExSizes, swExTones, swExRest,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
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
            <row gap="3" align="center">
              <if {switchDisabled()}>
                <if {switchTone()}>
                  <switch
                    bind:checked={notify}
                    size={switchSize()}
                    tone={switchTone()}
                    disabled
                  ></switch>
                {else =>
                  <switch bind:checked={notify} size={switchSize()} disabled></switch>}</if>
              {else =>
                <if {switchTone()}>
                  <switch
                    bind:checked={notify}
                    size={switchSize()}
                    tone={switchTone()}
                  ></switch>
                {else =>
                  <switch bind:checked={notify} size={switchSize()}></switch>}</if>}</if>
              <p style(monoXs)>{notify() ? 'on' : 'off'}</p>
            </row>
            <p style(monoXs)>{switchMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{switchSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>sm</span><span style(tick)>md</span><span style(tick)>lg</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Tone</label>
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="no tone — checked track paints primary" placement="top">
                <div label="rest" on:click={clearSwitchTone} selected={!switchTone() ? 'true' : null} style(swatch, swatchRest)></div>
              </tooltip>
              <tooltip content="primary" placement="top">
                <div label="primary" on:click={() => pickSwitchTone('primary')} selected={switchTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
              </tooltip>
              <tooltip content="success" placement="top">
                <div label="success" on:click={() => pickSwitchTone('success')} selected={switchTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
              </tooltip>
              <tooltip content="warning" placement="top">
                <div label="warning" on:click={() => pickSwitchTone('warning')} selected={switchTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickSwitchTone('destructive')} selected={switchTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
              <tooltip content="accent" placement="top">
                <div label="accent" on:click={() => pickSwitchTone('accent')} selected={switchTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
              </tooltip>
              <tooltip content="muted" placement="top">
                <div label="muted" on:click={() => pickSwitchTone('muted')} selected={switchTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
              </tooltip>
              <tooltip content="info" placement="top">
                <div label="info" on:click={() => pickSwitchTone('info')} selected={switchTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
              </tooltip>
            </row>
            <p style(mutedXs)>
              Tone paints the <strong>checked track</strong> only. Thumb always
              <code>component.switch.thumb</code> — never tone.foreground.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Checked</label>
            <row gap="2" align="center">
              <switch bind:checked={notify} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{notify() ? 'on' : 'off'}</span>
            </row>
            <p style(mutedXs)>
              <code>bind:checked</code> keeps the live specimen and a signal in lockstep.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Disabled</label>
            <row gap="2" align="center">
              <switch bind:checked={switchDisabled} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{switchDisabled() ? 'not-allowed' : 'interactive'}</span>
            </row>
            <p style(mutedXs)>
              Native <code>disabled</code> freezes interaction; PE keeps
              <code>component.switch.track</code> off paint and drops thumb shadow.
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>bind:checked</h3>
        <p style(exampleLead)>
          A native checkbox with <code>role=switch</code> — keyboard and forms come from
          the platform. <code>bind:checked</code> keeps the control and a signal in lockstep.
        </p>
        <row gap="3" align="center">
          <switch bind:checked={notify}></switch>
          <p style(mutedSm)>{notify() ? 'Notifications on' : 'Muted'}</p>
        </row>
        <code-block code={swExBind()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)>
          <code>sm</code> · <code>md</code> · <code>lg</code> scale track and thumb
          (default <code>md</code>).
        </p>
        <row gap="3" align="center">
          <switch size="sm" checked></switch>
          <switch checked></switch>
          <switch size="lg" checked></switch>
        </row>
        <code-block code={swExSizes()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Tone on the checked track</h3>
        <p style(exampleLead)>
          When <code>tone</code> is set, the on track uses that tone. The thumb never
          leaves <code>component.switch.thumb</code>.
        </p>
        <row gap="3" align="center">
          <switch tone="success" checked></switch>
          <switch tone="warning" checked></switch>
          <switch tone="destructive" checked></switch>
          <switch tone="info" checked></switch>
          <switch tone="muted" checked></switch>
        </row>
        <code-block code={swExTones()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Off track from <code>component.switch.track</code>; border from
          <code>component.switch.border</code>; thumb from
          <code>component.switch.thumb</code>. Checked without tone paints
          <code>primary</code>.
        </p>
        <row gap="3" align="center">
          <switch></switch>
          <switch checked></switch>
          <switch disabled></switch>
          <switch checked disabled></switch>
        </row>
        <code-block code={swExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
