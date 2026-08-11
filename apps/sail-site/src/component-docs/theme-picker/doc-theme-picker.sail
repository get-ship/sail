<!--
  <doc-theme-picker> — first-class docs host for /components/theme-picker.
  Specimen left, knobs right — value · inline · modeOnly (registry props only).
-->

<controller>
import type { Scope } from './doc-theme-picker.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const value = signal('')
  const inline = signal(false)
  const modeOnly = signal(false)
  const last = signal('')
  const onThemeChange = (detail: {
    theme?: string
    mode?: string
    preference?: string
  }): void => {
    const t = detail?.theme ?? ''
    if (t) value.set(t)
    last.set(
      [detail?.theme, detail?.mode, detail?.preference].filter(Boolean).join(' · ') || '',
    )
  }
  const tpMeta = derived(() => {
    const v = value() ? `value "${value()}"` : 'value (site default)'
    const i = inline() ? ' · inline' : ''
    const m = modeOnly() ? ' · modeOnly' : ''
    const l = last() ? ` · last ${last()}` : ''
    return `${v}${i}${m}${l} · component.theme-picker.*`
  })
  const entry = DETAILS['theme-picker']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const tpExDefault = () =>
    `<theme-picker></theme-picker>\n<!-- trigger → popover of live theme cards -->`
  const tpExValue = () =>
    `<theme-picker value="console" on-theme-change={onThemeChange}></theme-picker>`
  const tpExMode = () =>
    `<theme-picker mode-only></theme-picker>\n<!-- sun / moon / system cycle only -->`
  const tpExInline = () =>
    `<theme-picker inline></theme-picker>\n<!-- panel in-flow (no floating popover chrome) -->`
  return {
    value, inline, modeOnly, last, onThemeChange, tpMeta,
    heading, blurb, props, usageCode,
    tpExDefault, tpExValue, tpExMode, tpExInline,
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
  style stageCol = { box: w(panel-max-w-md).maxW(full) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style faceRow = { box: w(full) }
  style fullW = { box: w(full) }

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
          <column gap="3" align="center" style(stageCol)>
            <theme-picker
              value={value() || null}
              inline={inline() ? 'true' : null}
              mode-only={modeOnly() ? 'true' : null}
              on-theme-change={onThemeChange}
            ></theme-picker>
            <p style(monoXs)>{tpMeta()}</p>
            <p style(mutedXs)>Click the trigger to open the catalog panel. Swatches paint via real <code>theme=</code>.</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Value</label>
            <select size="sm" bind:value={value} label="Value">
              <option value="">(site default)</option>
              <option value="ship">ship</option>
              <option value="sail">sail</option>
              <option value="console">console</option>
              <option value="paper">paper</option>
            </select>
            <p style(mutedXs)>Active theme id, reflected after selection.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Inline</label>
            <row gap="2" align="center">
              <switch bind:checked={inline} size="sm" tone="primary" label="Inline"></switch>
              <span style(sizeValue)>{inline() ? 'in-flow panel' : 'popover'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Mode only</label>
            <row gap="2" align="center">
              <switch bind:checked={modeOnly} size="sm" tone="primary" label="Mode only"></switch>
              <span style(sizeValue)>{modeOnly() ? 'light/dark/system' : 'full catalog'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.theme-picker.bg / fg</code></p>
              <p style(mutedXs)><code>trigger-fg · option-fg · option-hover-bg</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Default popover</h3>
        <p style(exampleLead)>Trigger opens native popover of live theme specimens.</p>
        <theme-picker></theme-picker>
        <code-block code={tpExDefault()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Bound value</h3>
        <p style(exampleLead)><code>value</code> + <code>onThemeChange</code> for controlled selection.</p>
        <code-block code={tpExValue()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Mode only</h3>
        <p style(exampleLead)>Light / dark / system cycle — no full catalog.</p>
        <code-block code={tpExMode()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Inline</h3>
        <p style(exampleLead)>Panel in document flow (scaffold geometry only).</p>
        <code-block code={tpExInline()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
