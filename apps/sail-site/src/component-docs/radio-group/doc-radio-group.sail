<!--
  <doc-radio-group> — first-class docs host for /components/radio-group.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-radio-group.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const plan = signal('pro')
  const rgDisabled = signal(false)
  const rgRequired = signal(false)
  const rgLabel = signal('Plan')
  const lastChange = signal('')
  const onPlanChange = (detail: { value?: string }): void => {
    const v = detail?.value ?? ''
    plan.set(v)
    lastChange.set(v)
  }
  const rgMeta = derived(() => {
    const d = rgDisabled() ? 'disabled' : 'enabled'
    const r = rgRequired() ? 'required' : 'optional'
    const ch = lastChange() ? ` · last onChange ${lastChange()}` : ''
    return `value "${plan()}" · ${d} · ${r} · label "${rgLabel()}"${ch}`
  })
  const entry = DETAILS['radio-group']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const rgExBasic = () =>
    `<radio-group label="Plan" name="plan" value="pro" onChange={onChange}>\n  <radio value="free">Free</radio>\n  <radio value="pro">Pro</radio>\n  <radio value="enterprise">Enterprise</radio>\n</radio-group>`
  const rgExDisabled = () =>
    `<radio-group label="Plan" name="plan" value="pro" disabled>\n  <radio value="free">Free</radio>\n  <radio value="pro">Pro</radio>\n</radio-group>\n<!-- freezes every child radio input -->`
  const rgExRequired = () =>
    `<radio-group label="Plan" name="plan" value="" required>\n  <radio value="free">Free</radio>\n  <radio value="pro">Pro</radio>\n</radio-group>\n<!-- aria-required on the radiogroup host -->`
  return {
    plan, rgDisabled, rgRequired, rgLabel, lastChange, onPlanChange, rgMeta,
    heading, blurb, props, usageCode,
    rgExBasic, rgExDisabled, rgExRequired,
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
            <if {rgDisabled()}>
              <if {rgRequired()}>
                <radio-group
                  label={rgLabel()}
                  name="doc-rg-plan"
                  value={plan()}
                  required
                  disabled
                  onChange={onPlanChange}
                  style(stageCol)
                >
                  <radio value="free">Free</radio>
                  <radio value="pro">Pro</radio>
                  <radio value="enterprise">Enterprise</radio>
                </radio-group>
              {else =>
                <radio-group
                  label={rgLabel()}
                  name="doc-rg-plan"
                  value={plan()}
                  disabled
                  onChange={onPlanChange}
                  style(stageCol)
                >
                  <radio value="free">Free</radio>
                  <radio value="pro">Pro</radio>
                  <radio value="enterprise">Enterprise</radio>
                </radio-group>}</if>
            {else =>
              <if {rgRequired()}>
                <radio-group
                  label={rgLabel()}
                  name="doc-rg-plan"
                  value={plan()}
                  required
                  onChange={onPlanChange}
                  style(stageCol)
                >
                  <radio value="free">Free</radio>
                  <radio value="pro">Pro</radio>
                  <radio value="enterprise">Enterprise</radio>
                </radio-group>
              {else =>
                <radio-group
                  label={rgLabel()}
                  name="doc-rg-plan"
                  value={plan()}
                  onChange={onPlanChange}
                  style(stageCol)
                >
                  <radio value="free">Free</radio>
                  <radio value="pro">Pro</radio>
                  <radio value="enterprise">Enterprise</radio>
                </radio-group>}</if>}</if>
            <p style(monoXs)>{rgMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Value</label>
            <select size="sm" bind:value={plan} label="Value">
              <option value="free">free</option>
              <option value="pro">pro</option>
              <option value="enterprise">enterprise</option>
            </select>
            <p style(mutedXs)>
              Selected option value. Clicking a radio also updates via
              <code>onChange</code>.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Label</label>
            <select size="sm" bind:value={rgLabel} label="Label">
              <option value="Plan">Plan</option>
              <option value="Billing period">Billing period</option>
              <option value="Tier">Tier</option>
            </select>
            <p style(mutedXs)>
              Accessible group name → <code>aria-label</code> on the radiogroup host.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Disabled</label>
            <row gap="2" align="center">
              <switch bind:checked={rgDisabled} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{rgDisabled() ? 'frozen' : 'interactive'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Required</label>
            <row gap="2" align="center">
              <switch bind:checked={rgRequired} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{rgRequired() ? 'aria-required' : 'optional'}</span>
            </row>
          </column>
          <column gap="2">
            <p style(mutedXs)>
              Host is layout + typeface only — no
              <code>component.radio-group.*</code> leaves. Option paint is child
              <code>&lt;radio&gt;</code> (<code>component.radio.fg</code>).
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Grouped options</h3>
        <p style(exampleLead)>
          One value, one name, real <code>&lt;radio&gt;</code> children. The platform
          owns exclusivity and keyboard.
        </p>
        <radio-group label="Plan" name="ex-rg-basic" value="pro" style(stageCol)>
          <radio value="free">Free</radio>
          <radio value="pro">Pro</radio>
          <radio value="enterprise">Enterprise</radio>
        </radio-group>
        <code-block code={rgExBasic()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Disabled group</h3>
        <p style(exampleLead)>
          Group <code>disabled</code> freezes every child radio input (group OR item).
        </p>
        <radio-group label="Plan" name="ex-rg-dis" value="pro" disabled style(stageCol)>
          <radio value="free">Free</radio>
          <radio value="pro">Pro</radio>
        </radio-group>
        <code-block code={rgExDisabled()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Required</h3>
        <p style(exampleLead)>
          <code>required</code> stamps <code>aria-required</code> on the radiogroup host.
        </p>
        <radio-group label="Plan" name="ex-rg-req" value="free" required style(stageCol)>
          <radio value="free">Free</radio>
          <radio value="pro">Pro</radio>
        </radio-group>
        <code-block code={rgExRequired()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
