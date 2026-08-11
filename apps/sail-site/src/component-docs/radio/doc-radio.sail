<!--
  <doc-radio> — first-class docs host for /components/radio.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-radio.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const radioDisabled = signal(false)
  const radioValue = signal('monthly')
  const radioMeta = derived(() => {
    return `value "${radioValue()}" · disabled ${radioDisabled() ? 'on' : 'off'} · component.radio.fg`
  })
  const entry = DETAILS.radio
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const radioExOption = () =>
    `<radio value="monthly">Monthly</radio>\n<radio value="yearly">Yearly</radio>`
  const radioExDisabled = () =>
    `<radio value="enterprise" disabled>Enterprise (locked)</radio>\n<!-- host dims; radio-group also sets input.disabled -->`
  const radioExRest = () =>
    `<radio value="a">Option A</radio>\n<!-- label ink → component.radio.fg (--radio-foreground) -->`
  const radioExGroup = () =>
    `<radio-group label="Plan" name="plan" value="monthly">\n  <radio value="monthly">Monthly</radio>\n  <radio value="yearly">Yearly</radio>\n</radio-group>\n<!-- exclusivity / name / onChange — radio-group (separate L1) -->`
  return {
    radioDisabled, radioValue, radioMeta,
    heading, blurb, props, usageCode,
    radioExOption, radioExDisabled, radioExRest, radioExGroup,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageSplit = { box: w(full).minW(0).minH(64) }
  style stageCanvas = { box: grow(1).basis(0).minW(0).minH(64).pad(8) }
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
          <column gap="4" align="center" style(stageCol)>
            <column gap="2" style(stageCol)>
              <if {radioDisabled()}>
                <radio value={radioValue()} disabled>Live option</radio>
              {else =>
                <radio value={radioValue()}>Live option</radio>}</if>
              <radio value="other" disabled>Always disabled peer</radio>
            </column>
            <p style(monoXs)>{radioMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Value</label>
            <select size="sm" bind:value={radioValue} label="Value">
              <option value="monthly">monthly</option>
              <option value="yearly">yearly</option>
              <option value="enterprise">enterprise</option>
            </select>
            <p style(mutedXs)>
              This option’s <code>value</code> string. Inside
              <code>radio-group</code>, the group selects the child whose value
              matches the group value.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Disabled</label>
            <row gap="2" align="center">
              <switch bind:checked={radioDisabled} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{radioDisabled() ? 'on' : 'off'}</span>
            </row>
            <p style(mutedXs)>
              Host attribute for <code>radio-group</code> composition —
              the group sets nested <code>input.disabled</code> so Style PE dims
              the control. Grouping is a separate L1.
            </p>
          </column>
          <column gap="2">
            <p style(mutedXs)>
              Label ink from <code>component.radio.fg</code>
              (<code>--radio-foreground</code>). No size or tone prop.
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Option unit</h3>
        <p style(exampleLead)>
          Each <code>&lt;radio&gt;</code> is one labelled option — native radio inside
          a label host. <code>value</code> is this option’s value string.
        </p>
        <column gap="2" style(stageCol)>
          <radio value="monthly">Monthly</radio>
          <radio value="yearly">Yearly</radio>
        </column>
        <code-block code={radioExOption()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Disabled option</h3>
        <p style(exampleLead)>
          <code>disabled</code> is a host attribute. Compose with
          <code>radio-group</code> so the nested input is disabled for platform
          interaction and Style PE paint.
        </p>
        <column gap="2" style(stageCol)>
          <radio value="enterprise" disabled>Enterprise (locked)</radio>
        </column>
        <code-block code={radioExDisabled()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Label ink always rides <code>component.radio.fg</code>. No tone or size
          axis — override that leaf in Studio.
        </p>
        <column gap="2" style(stageCol)>
          <radio value="a">Option A</radio>
        </column>
        <code-block code={radioExRest()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>With radio-group (composition)</h3>
        <p style(exampleLead)>
          Shared name, exclusivity, keyboard, and <code>onChange</code> are owned by
          <code>radio-group</code> — a separate L1.
        </p>
        <radio-group label="Plan" name="ex-radio-plan" value="monthly" style(stageCol)>
          <radio value="monthly">Monthly</radio>
          <radio value="yearly">Yearly</radio>
        </radio-group>
        <code-block code={radioExGroup()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
