<!--
  <doc-theme-picker> — /components/theme-picker
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style specimen = { box: size({ w: full, max-w: full }).space({ pad: 2 }).overflow({ all: visible }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style sizeValue = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
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
      <doc-example
        title="Default popover"
        lead="Trigger opens a panel of live theme specimens."
        code={exDefault()}
      >
        <theme-picker></theme-picker>
      </doc-example>
      <doc-example
        title="Selected theme"
        lead="value is the selected theme id — it applies that theme."
        code={exValue()}
      >
        <theme-picker value="console" on-theme-change={onThemeChange}></theme-picker>
      </doc-example>
      <doc-example
        title="Mode only"
        lead="mode-only cycles light / dark / system. No catalog."
        code={exMode()}
      >
        <theme-picker mode-only></theme-picker>
      </doc-example>
      <doc-example
        title="Inline"
        lead="Catalog stays in the page — no popover, no hidden trigger."
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
