<!--
  <doc-toggle-group> — /components/toggle-group
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(full).maxW(full).pad(2).overflow(visible) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
</scope>

<controller>
import type { Scope } from './doc-toggle-group.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const tgType = signal('single')
  const tgVariant = signal('outline')
  const tgOrientation = signal('horizontal')
  const tgValue = signal('day')
  const lastChange = signal('')
  const onToggleChange = (detail: { value?: string }): void => {
    const v = detail?.value ?? ''
    tgValue.set(v)
    lastChange.set(v)
  }

  const entry = DETAILS['toggle-group']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    return `<toggle-group type="${tgType()}" value="${tgValue()}" variant="${tgVariant()}" orientation="${tgOrientation()}" on-toggle-change={onChange}>\n  <toggle value="day">Day</toggle>\n  <toggle value="week">Week</toggle>\n  <toggle value="month">Month</toggle>\n</toggle-group>`
  })

  const exSingle = () =>
    `<toggle-group type="single" value="day" variant="outline" on-toggle-change={onChange}>\n  <toggle value="day">Day</toggle>\n  <toggle value="week">Week</toggle>\n  <toggle value="month">Month</toggle>\n</toggle-group>`
  const exSegmented = () =>
    `<toggle-group variant="segmented" value="list">\n  <toggle value="grid">Grid</toggle>\n  <toggle value="list">List</toggle>\n  <toggle value="board">Board</toggle>\n</toggle-group>`
  const exMulti = () =>
    `<toggle-group type="multiple" value="grid board">\n  <toggle value="grid">Grid</toggle>\n  <toggle value="list">List</toggle>\n  <toggle value="board">Board</toggle>\n</toggle-group>`

  return {
    tgType,
    tgVariant,
    tgOrientation,
    tgValue,
    lastChange,
    onToggleChange,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exSingle,
    exSegmented,
    exMulti,
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
      <toggle-group
        type={tgType()}
        value={tgValue()}
        variant={tgVariant()}
        orientation={tgOrientation()}
        on-toggle-change={onToggleChange}
        label="Range"
      >
        <toggle value="day">Day</toggle>
        <toggle value="week">Week</toggle>
        <toggle value="month">Month</toggle>
      </toggle-group>
      <if {lastChange()}>
        <span style(sizeValue)>Selected: {lastChange()}</span>
      </if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Type</label>
        <select size="sm" bind:value={tgType} label="Type">
          <option value="single">single</option>
          <option value="multiple">multiple</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Variant</label>
        <select size="sm" bind:value={tgVariant} label="Variant">
          <option value="outline">outline</option>
          <option value="ghost">ghost</option>
          <option value="segmented">segmented</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Orientation</label>
        <select size="sm" bind:value={tgOrientation} label="Orientation">
          <option value="horizontal">horizontal</option>
          <option value="vertical">vertical</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Value</label>
        <select size="sm" bind:value={tgValue} label="Value">
          <option value="day">day</option>
          <option value="week">week</option>
          <option value="month">month</option>
          <option value="day week">day week</option>
          <option value="day month">day month</option>
          <option value="week month">week month</option>
          <option value="day week month">day week month</option>
        </select>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(specimen)>
      <toggle-group type="single" value="day" variant="outline" on-toggle-change={onToggleChange}>
        <toggle value="day">Day</toggle>
        <toggle value="week">Week</toggle>
        <toggle value="month">Month</toggle>
      </toggle-group>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Single selection"
        lead="One value at a time. Arrow keys move focus across the group."
        code={exSingle()}
      >
        <toggle-group type="single" value="day" variant="outline" on-toggle-change={onToggleChange}>
          <toggle value="day">Day</toggle>
          <toggle value="week">Week</toggle>
          <toggle value="month">Month</toggle>
        </toggle-group>
      </doc-example>
      <doc-example
        title="Segmented rail"
        lead='variant="segmented" — the connected control look.'
        code={exSegmented()}
      >
        <toggle-group variant="segmented" value="list" on-toggle-change={onToggleChange}>
          <toggle value="grid">Grid</toggle>
          <toggle value="list">List</toggle>
          <toggle value="board">Board</toggle>
        </toggle-group>
      </doc-example>
      <doc-example
        title="Multiple"
        lead='type="multiple" accumulates a space-joined value set.'
        code={exMulti()}
      >
        <toggle-group type="multiple" value="grid board" on-toggle-change={onToggleChange}>
          <toggle value="grid">Grid</toggle>
          <toggle value="list">List</toggle>
          <toggle value="board">Board</toggle>
        </toggle-group>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/switch">Switch</a>
  </row>
</column>
