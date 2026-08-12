<!--
  <doc-theme-picker> — /components/theme-picker
  Shared chrome: split + snippet copy + usage Code|Live|Split.
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

  const entry = DETAILS['theme-picker']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (value()) a.push(`value="${value()}"`)
    if (inline()) a.push('inline')
    if (modeOnly()) a.push('mode-only')
    a.push('on-theme-change={onThemeChange}')
    return a.length ? `<theme-picker ${a.join(' ')}></theme-picker>` : `<theme-picker></theme-picker>`
  })

  const exDefault = () => `<theme-picker></theme-picker>`
  const exValue = () =>
    `<theme-picker value="console" on-theme-change={onThemeChange}></theme-picker>`
  const exMode = () => `<theme-picker mode-only></theme-picker>`
  const exInline = () => `<theme-picker inline></theme-picker>`

  return {
    value,
    inline,
    modeOnly,
    last,
    onThemeChange,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exDefault,
    exValue,
    exMode,
    exInline,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(full).maxW(full).pad(2).overflow(visible) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style exampleSection = { box: w(full).padY(4).overflow(visible) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
</scope>

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
    <column slot="specimen" gap="3" align="center" style(specimen)>
      <theme-picker
        value={value() || null}
        inline={inline() ? 'true' : null}
        mode-only={modeOnly() ? 'true' : null}
        on-theme-change={onThemeChange}
      ></theme-picker>
      <if {last()}>
        <span style(sizeValue)>{last()}</span>
      </if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Value</label>
        <select size="sm" bind:value={value} label="Value">
          <option value="">(site default)</option>
          <option value="ship">ship</option>
          <option value="sail">sail</option>
          <option value="console">console</option>
          <option value="paper">paper</option>
        </select>
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={inline} size="sm" /> Inline</label>
        <label><checkbox bind:checked={modeOnly} size="sm" /> Mode only</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(specimen)>
      <theme-picker></theme-picker>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="3" style(exampleSection)>
        <h3 style(exampleH)>Default popover</h3>
        <p style(exampleLead)>Trigger opens a panel of live theme specimens.</p>
        <theme-picker></theme-picker>
        <doc-code code={exDefault()} lang="sail"></doc-code>
      </column>
      <column gap="3" style(exampleSection)>
        <h3 style(exampleH)>Bound value</h3>
        <p style(exampleLead)><code>value</code> + <code>onThemeChange</code> for controlled selection.</p>
        <doc-code code={exValue()} lang="sail"></doc-code>
      </column>
      <column gap="3" style(exampleSection)>
        <h3 style(exampleH)>Mode only</h3>
        <p style(exampleLead)>Light / dark / system cycle — no full catalog.</p>
        <theme-picker mode-only></theme-picker>
        <doc-code code={exMode()} lang="sail"></doc-code>
      </column>
      <column gap="3" style(exampleSection)>
        <h3 style(exampleH)>Inline</h3>
        <p style(exampleLead)>Panel in document flow (no floating chrome).</p>
        <theme-picker inline></theme-picker>
        <doc-code code={exInline()} lang="sail"></doc-code>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components">All components</a>
  </row>
</column>
