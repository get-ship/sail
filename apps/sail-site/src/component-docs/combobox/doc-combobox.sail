<!--
  <doc-combobox> — /components/combobox
  Fruits demo. Every surface (specimen, snippet, knobs, usage, live, examples)
  shows the same four options. Width is content-sized, not full-bleed.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  /** Parent width constraint — product host is size({ w: 100% }) of its parent. */
  style controlCol = { size({ w: 56, max-w: 100% }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-combobox.sail'
import { DETAILS } from '../../compositions/component-details'

const FRUITS = [
  { value: 'apple', label: 'Apple' },
  { value: 'banana', label: 'Banana' },
  { value: 'cherry', label: 'Cherry' },
  { value: 'grape', label: 'Grape' },
] as const

export default controller((_ctx: Scope) => {
  const placeholder = signal('Pick a fruit')
  const value = signal('')
  const last = signal('')
  const onSelect = (detail: { value?: string; label?: string }): void => {
    value.set(detail?.value ?? '')
    last.set(detail?.label ?? detail?.value ?? '')
  }

  const entry = DETAILS.combobox
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const optionsBlock = (): string =>
    FRUITS.map((f) => `  <option value="${f.value}">${f.label}</option>`).join('\n')

  const specimenCode = derived(() => {
    const a: string[] = []
    if (placeholder()) a.push(`placeholder="${placeholder()}"`)
    if (value()) a.push(`value="${value()}"`)
    a.push('on-select={onSelect}')
    return `<combobox ${a.join(' ')}>\n${optionsBlock()}\n</combobox>`
  })

  const exBasic = () =>
    `<combobox placeholder="Pick a fruit" on-select={onSelect}>\n${optionsBlock()}\n</combobox>`

  return {
    placeholder,
    value,
    last,
    onSelect,
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
    <column slot="specimen" gap="3" align="center" style(controlCol)>
      <combobox
        placeholder={placeholder()}
        value={value()}
        on-select={onSelect}
      >
        <option value="apple">Apple</option>
        <option value="banana">Banana</option>
        <option value="cherry">Cherry</option>
        <option value="grape">Grape</option>
      </combobox>
      <span style(sizeValue)>Picked: {last() || 'nothing yet'}</span>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Placeholder</label>
        <input type="text" size="sm" bind:value={placeholder} />
      </column>
      <column gap="2">
        <label style(knobLabel)>Value</label>
        <select size="sm" bind:value={value} label="Value">
          <option value="">(none)</option>
          <option value="apple">apple</option>
          <option value="banana">banana</option>
          <option value="cherry">cherry</option>
          <option value="grape">grape</option>
        </select>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(controlCol)>
      <combobox placeholder="Pick a fruit" on-select={onSelect}>
        <option value="apple">Apple</option>
        <option value="banana">Banana</option>
        <option value="cherry">Cherry</option>
        <option value="grape">Grape</option>
      </combobox>
      <p style(sizeValue)>Picked: {last() || 'nothing yet'}</p>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Type to filter"
        lead="Arrows move, Enter commits, Escape closes."
        code={exBasic()}
      >
        <column style(controlCol)>
          <combobox placeholder="Pick a fruit" on-select={onSelect}>
            <option value="apple">Apple</option>
            <option value="banana">Banana</option>
            <option value="cherry">Cherry</option>
            <option value="grape">Grape</option>
          </combobox>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/select">Select</a>
  </row>
</column>
