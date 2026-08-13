<!--
  <doc-date> — /components/date
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style fieldShell = { box: size({ w: full, max-w: 64 }).space({ pad: 2 }).overflow({ all: visible }) }
  style fullW = { box: size({ w: full }).overflow({ all: visible }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style sizeValue = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style tickRow = { box: size({ w: full }).space({ pad-x: 1_5, pad-t: 1 }) }
  style tick = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
</scope>

<controller>
import type { Scope } from './doc-date.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SIZES = ['xs', 'sm', 'md', 'lg'] as const
  const sizeIdx = signal(2)
  const precision = signal('date')
  const invalid = signal(false)
  const disabled = signal(false)
  const value = signal('')

  const size = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SIZES.length - 1)
    return SIZES[i] ?? 'md'
  })

  const entry = DETAILS.date
  const heading = () => entry?.heading ?? '<date>'
  const blurb = () => entry?.blurb ?? ''
  const props = () => entry?.props ?? []
  const usageCode = () => entry?.usage ?? ''

  const specimenCode = derived(() => {
    const a: string[] = [`type="${precision()}"`]
    if (size() !== 'md') a.push(`size="${size()}"`)
    if (invalid()) a.push('invalid')
    if (disabled()) a.push('disabled')
    a.push('bind:value={value}')
    return `<date ${a.join(' ')} />`
  })

  const exTypes = () =>
    `<date type="date" />\n<date type="time" step="900" />\n<date type="datetime-local" />\n<date type="month" />\n<date type="week" />`

  return {
    sizeIdx,
    size,
    precision,
    invalid,
    disabled,
    value,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exTypes,
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
    <column slot="specimen" gap="3" style(fieldShell)>
      <date
        type={precision()}
        size={size()}
        invalid={invalid()}
        disabled={disabled()}
        bind:value={value}
        style(fullW)
     />
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Type</label>
        <select size="sm" bind:value={precision} label="Type">
          <option value="date">date</option>
          <option value="time">time</option>
          <option value="datetime-local">datetime-local</option>
          <option value="month">month</option>
          <option value="week">week</option>
        </select>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{size()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="3" step="1" size="sm" label="Size" ticks="xs,sm,md,lg"></slider>
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={invalid} size="sm" /> Invalid</label>
        <label><checkbox bind:checked={disabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" style(fieldShell)>
      <date type="date" style(fullW) />
      <date type="time" step="900" style(fullW) />
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Precision"
        lead="Pick the shape with type. Use step when increments matter (for example, 15-minute times)."
        code={exTypes()}
      >
        <column gap="2" style(fieldShell)>
          <date type="date" style(fullW) />
          <date type="time" step="900" style(fullW) />
          <date type="datetime-local" style(fullW) />
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/input">Input</a>
  </row>
</column>
