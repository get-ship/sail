<!--
  <doc-fieldset> — /components/fieldset
  Owns <legend> documentation (no top-level legend hub tile).
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<controller>
import type { Scope } from './doc-fieldset.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const fieldsetDisabled = signal(false)

  const entry = DETAILS.fieldset
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const d = fieldsetDisabled() ? ' disabled' : ''
    return `<fieldset${d}>\n  <legend>Account details</legend>\n  <column gap="2">\n    <input placeholder="Name" />\n    <input placeholder="Email" />\n  </column>\n</fieldset>`
  })

  const exGroup = () =>
    `<fieldset>\n  <legend>Account details</legend>\n  <column gap="2">\n    <input placeholder="Name" />\n    <input placeholder="Email" />\n  </column>\n</fieldset>`
  const exDisabled = () =>
    `<fieldset disabled>\n  <legend>Billing (locked)</legend>\n  <column gap="2">\n    <input placeholder="Card" />\n    <button type="button" variant="solid" size="sm">Save</button>\n  </column>\n</fieldset>`

  return {
    fieldsetDisabled,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exGroup,
    exDisabled,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(64).maxW(full).pad(2).overflow(visible) }
  style controlW = { box: w(64).maxW(full) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style exampleSection = { box: w(full).padY(4).overflow(visible) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
</scope>

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
      <if {fieldsetDisabled()}>
        <fieldset disabled style(controlW)>
          <legend>Account details</legend>
          <column gap="2">
            <input placeholder="Name" />
            <input placeholder="Email" />
            <button type="button" variant="solid" size="sm">Save</button>
          </column>
        </fieldset>
      {else =>
        <fieldset style(controlW)>
          <legend>Account details</legend>
          <column gap="2">
            <input placeholder="Name" />
            <input placeholder="Email" />
            <button type="button" variant="solid" size="sm">Save</button>
          </column>
        </fieldset>}</if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <row gap="4" align="center">
        <label><checkbox bind:checked={fieldsetDisabled} size="sm" /> Disabled</label>
      </row>
      <p style(exampleLead)>
        Disabled freezes every control inside and dims the group.
      </p>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(specimen)>
      <fieldset style(controlW)>
        <legend>Account details</legend>
        <column gap="2">
          <input placeholder="Name" />
          <input placeholder="Email" />
        </column>
      </fieldset>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="3" style(exampleSection)>
        <h3 style(exampleH)>Group with legend</h3>
        <p style(exampleLead)>
          Put a <code>legend</code> first, then related controls. Legend is documented
          here — there is no separate hub page for it.
        </p>
        <fieldset style(controlW)>
          <legend>Account details</legend>
          <column gap="2">
            <input placeholder="Name" />
            <input placeholder="Email" />
          </column>
        </fieldset>
        <doc-code code={exGroup()} lang="sail"></doc-code>
      </column>
      <column gap="3" style(exampleSection)>
        <h3 style(exampleH)>Disabled set</h3>
        <p style(exampleLead)>
          <code>disabled</code> freezes every control inside; the legend still names the group.
        </p>
        <fieldset disabled style(controlW)>
          <legend>Billing (locked)</legend>
          <column gap="2">
            <input placeholder="Card" />
            <button type="button" variant="solid" size="sm">Save</button>
          </column>
        </fieldset>
        <doc-code code={exDisabled()} lang="sail"></doc-code>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/field">Field</a>
  </row>
</column>
