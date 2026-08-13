<!--
  <doc-color> — /components/color
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style fieldShell = { box: size({ w: full, max-w: 64 }).space({ pad: 2 }).overflow({ all: visible }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
  style demoRow = { box: size({ w: full }).space({ pad-y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-color.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const value = signal('#c8f542')
  const invalid = signal(false)
  const disabled = signal(false)

  const entry = DETAILS.color
  const heading = () => entry?.heading ?? '<color>'
  const blurb = () => entry?.blurb ?? ''
  const props = () => entry?.props ?? []
  const usageCode = () => entry?.usage ?? ''

  const specimenCode = derived(() => {
    const a: string[] = [`value="${value()}"`]
    if (invalid()) a.push('invalid')
    if (disabled()) a.push('disabled')
    return `<color ${a.join(' ')} />`
  })

  const exBasic = () => `<color value="#c8f542" />\n<color bind:value={hex} />`

  return {
    value,
    invalid,
    disabled,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exBasic,
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
    <column slot="specimen" gap="3" align="start" style(fieldShell)>
      <color bind:value={value} invalid={invalid()} disabled={disabled()} />
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Value</label>
        <input type="text" size="sm" bind:value={value} placeholder="#rrggbb" />
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={invalid} size="sm" /> Invalid</label>
        <label><checkbox bind:checked={disabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(fieldShell)>
      <color value="#c8f542" />
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="System swatch"
        lead="One field for a hex colour. For full HSL channels, use the colour-picker package."
        code={exBasic()}
      >
        <row gap="3" align="center" style(demoRow)>
          <color value="#c8f542" />
          <color value="#3b82f6" />
          <color value="#ef4444" />
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/input">Input</a>
  </row>
</column>
