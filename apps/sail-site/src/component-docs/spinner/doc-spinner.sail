<!--
  <doc-spinner> — first-class docs host for /components/spinner.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-spinner.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {

  const SPN_SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const spinnerTone = signal('primary')
  const spinnerCentered = signal(false)
  const pickSpinnerTone = (t: string): void => spinnerTone.set(t)
  const clearSpinnerTone = (): void => spinnerTone.set('')
  const spinnerSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SPN_SIZES.length - 1)
    return SPN_SIZES[i] ?? 'md'
  })
  const spinnerMeta = derived(() => {
    const t = spinnerTone()
    return `${spinnerSize()} · ${t || 'rest (spinner.foreground)'} · centered ${spinnerCentered() ? 'on' : 'off'}`
  })
  const entry = DETAILS.spinner
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const spnExInline = () =>
    `<row gap="2" align="center">\n  <spinner size="sm" tone="primary"></spinner>\n  <p>fetching…</p>\n</row>`
  const spnExSizes = () =>
    `<spinner size="xs" tone="primary"></spinner>\n<spinner size="sm" tone="primary"></spinner>\n<spinner tone="primary"></spinner>\n<spinner size="lg" tone="primary"></spinner>`
  const spnExRest = () =>
    `<spinner></spinner>\n<!-- no tone → component.spinner.fg + spinner.track -->`
  const spnExCentered = () =>
    `<spinner centered tone="primary"></spinner>`
  return {
    sizeIdx, spinnerTone, spinnerCentered, spinnerSize, spinnerMeta,
    pickSpinnerTone, clearSpinnerTone,
    heading, blurb, props, usageCode,
    spnExInline, spnExSizes, spnExRest, spnExCentered,
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
          <column gap="4" align="center" style(fullW)>
            <if {spinnerTone()}>
              <spinner size={spinnerSize()} tone={spinnerTone()} centered={spinnerCentered() ? 'true' : null}></spinner>
            {else =>
              <spinner size={spinnerSize()} centered={spinnerCentered() ? 'true' : null}></spinner>}</if>
            <p style(monoXs)>{spinnerMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{spinnerSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="3" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>xs</span><span style(tick)>sm</span><span style(tick)>md</span><span style(tick)>lg</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Tone</label>
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="rest — component.spinner.fg" placement="top">
                <div label="rest" on:click={clearSpinnerTone} selected={!spinnerTone() ? 'true' : null} style(swatch, swatchRest)></div>
              </tooltip>
              <tooltip content="primary" placement="top">
                <div label="primary" on:click={() => pickSpinnerTone('primary')} selected={spinnerTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
              </tooltip>
              <tooltip content="info" placement="top">
                <div label="info" on:click={() => pickSpinnerTone('info')} selected={spinnerTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
              </tooltip>
              <tooltip content="success" placement="top">
                <div label="success" on:click={() => pickSpinnerTone('success')} selected={spinnerTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
              </tooltip>
              <tooltip content="warning" placement="top">
                <div label="warning" on:click={() => pickSpinnerTone('warning')} selected={spinnerTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickSpinnerTone('destructive')} selected={spinnerTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
              <tooltip content="accent" placement="top">
                <div label="accent" on:click={() => pickSpinnerTone('accent')} selected={spinnerTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
              </tooltip>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Centered</label>
            <row gap="2" align="center">
              <switch bind:checked={spinnerCentered} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{spinnerCentered() ? 'block + mx auto' : 'inline'}</span>
            </row>
          </column>
        </column>
      </row>
    </column>
    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Inline with status copy</h3>
        <p style(exampleLead)>Pair a small spinner with a short wait message.</p>
        <row gap="2" align="center">
          <spinner size="sm" tone="primary"></spinner>
          <p style(mutedSm)>fetching…</p>
        </row>
        <code-block code={spnExInline()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Size ladder</h3>
        <p style(exampleLead)><code>xs</code> · <code>sm</code> · <code>md</code> · <code>lg</code>.</p>
        <row gap="4" align="center">
          <spinner size="xs" tone="primary"></spinner>
          <spinner size="sm" tone="primary"></spinner>
          <spinner tone="primary"></spinner>
          <spinner size="lg" tone="primary"></spinner>
        </row>
        <code-block code={spnExSizes()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>No <code>tone</code> → <code>component.spinner.fg</code> + track.</p>
        <row gap="4" align="center">
          <spinner></spinner>
          <spinner size="lg"></spinner>
        </row>
        <code-block code={spnExRest()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Centered in a container</h3>
        <p style(exampleLead)><code>centered</code> makes the host a block with auto horizontal margins.</p>
        <column gap="2" style(centerBox)>
          <spinner centered tone="primary"></spinner>
        </column>
        <code-block code={spnExCentered()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
