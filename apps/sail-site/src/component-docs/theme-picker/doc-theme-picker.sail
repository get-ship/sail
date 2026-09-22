<!--
  <doc-theme-picker> — /components/theme-picker
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 100%, max-w: 100% }).pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type(label) }
  style sizeValue = { fg(muted-foreground).type(mono-sm) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type(label) }
</scope>

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
        <label style(knobLabel)>Selected theme</label>
        <select size="sm" bind:value={value} label="Value">
          <option value="">(site default)</option>
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
      <doc-example
        title="Menu"
        lead="A compact control. Click it to browse themes and see how each one looks."
        code={exDefault()}
      >
        <theme-picker></theme-picker>
      </doc-example>
      <doc-example
        title="Start on a theme"
        lead="value picks the theme that is already on."
        code={exValue()}
      >
        <theme-picker value="console" on-theme-change={onThemeChange}></theme-picker>
      </doc-example>
      <doc-example
        title="Light, dark, or system"
        lead="Just the three mode buttons — for when the theme is already chosen."
        code={exMode()}
      >
        <theme-picker mode-only></theme-picker>
      </doc-example>
      <doc-example
        title="Always on the page"
        lead="Puts the theme list in the layout. Use this on a settings screen."
        code={exInline()}
      >
        <theme-picker inline></theme-picker>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components">All components</a>
  </row>
</column>
