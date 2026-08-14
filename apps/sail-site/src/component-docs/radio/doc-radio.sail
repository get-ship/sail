<!--
  <doc-radio> — /components/radio
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 56, max-w: full }).pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-radio.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const disabled = signal(false)
  const radioValue = signal('monthly')

  const entry = DETAILS.radio
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`value="${radioValue()}"`]
    if (disabled()) a.push('disabled')
    return `<radio ${a.join(' ')}>Live option</radio>`
  })

  const exOptions = () =>
    `<radio value="monthly">Monthly</radio>\n<radio value="yearly">Yearly</radio>`
  const exGroup = () =>
    `<radio-group label="Plan" name="plan" value="monthly">\n  <radio value="monthly">Monthly</radio>\n  <radio value="yearly">Yearly</radio>\n</radio-group>`

  return {
    disabled,
    radioValue,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exOptions,
    exGroup,
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
    <column slot="specimen" gap="3" align="start" style(specimen)>
      <if {disabled()}>
        <radio value={radioValue()} disabled>Live option</radio>
      {else =>
        <radio value={radioValue()}>Live option</radio>}</if>
      <radio value="other" disabled>Always disabled peer</radio>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Value</label>
        <select size="sm" bind:value={radioValue} label="Value">
          <option value="monthly">monthly</option>
          <option value="yearly">yearly</option>
          <option value="enterprise">enterprise</option>
        </select>
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={disabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(specimen)>
      <radio-group label="Plan" name="ex-radio-plan" value="monthly">
        <radio value="monthly">Monthly</radio>
        <radio value="yearly">Yearly</radio>
      </radio-group>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Options"
        lead="Prefer radio-group for exclusive choice."
        code={exOptions()}
      >
        <column gap="2">
          <radio value="monthly">Monthly</radio>
          <radio value="yearly">Yearly</radio>
        </column>
      </doc-example>
      <doc-example
        title="In a group"
        code={exGroup()}
      >
        <radio-group label="Plan" name="ex-rg-docs" value="monthly">
          <radio value="monthly">Monthly</radio>
          <radio value="yearly">Yearly</radio>
        </radio-group>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/radio-group">Radio group</a>
  </row>
</column>
