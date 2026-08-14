<!--
  <doc-radio-group> — /components/radio-group
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 56, max-w: full }).pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-radio-group.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const plan = signal('pro')
  const disabled = signal(false)
  const required = signal(false)
  const label = signal('Plan')
  const onPlanChange = (detail: { value?: string }): void => {
    plan.set(detail?.value ?? '')
  }

  const entry = DETAILS['radio-group']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [
      `label="${label()}"`,
      'name="plan"',
      `value="${plan()}"`,
    ]
    if (disabled()) a.push('disabled')
    if (required()) a.push('required')
    return `<radio-group ${a.join(' ')}>\n  <radio value="free">Free</radio>\n  <radio value="pro">Pro</radio>\n  <radio value="enterprise">Enterprise</radio>\n</radio-group>`
  })

  const exBasic = () =>
    `<radio-group label="Plan" name="plan" value="pro">\n  <radio value="free">Free</radio>\n  <radio value="pro">Pro</radio>\n  <radio value="enterprise">Enterprise</radio>\n</radio-group>`

  return {
    plan,
    disabled,
    required,
    label,
    onPlanChange,
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
    <column slot="specimen" gap="3" align="start" style(specimen)>
      <radio-group
        label={label()}
        name="doc-rg-plan"
        value={plan()}
        required={required()}
        disabled={disabled()}
        onChange={onPlanChange}
      >
        <radio value="free">Free</radio>
        <radio value="pro">Pro</radio>
        <radio value="enterprise">Enterprise</radio>
      </radio-group>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Label</label>
        <input type="text" size="sm" bind:value={label} />
      </column>
      <row gap="4" align="center">
        <label><checkbox bind:checked={required} size="sm" /> Required</label>
        <label><checkbox bind:checked={disabled} size="sm" /> Disabled</label>
      </row>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(specimen)>
      <radio-group label="Plan" name="ex-rg-basic" value="pro">
        <radio value="free">Free</radio>
        <radio value="pro">Pro</radio>
        <radio value="enterprise">Enterprise</radio>
      </radio-group>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Basic"
        lead="One shared value and accessible name for the set."
        code={exBasic()}
      >
        <radio-group label="Plan" name="ex-rg-docs" value="pro">
          <radio value="free">Free</radio>
          <radio value="pro">Pro</radio>
          <radio value="enterprise">Enterprise</radio>
        </radio-group>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/radio">Radio</a>
  </row>
</column>
