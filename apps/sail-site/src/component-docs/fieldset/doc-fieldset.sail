<!--
  <doc-fieldset> — /components/fieldset
  Owns <legend> documentation (no top-level legend hub tile).
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 64, max-w: full }).pad(2).overflow({ all: visible }) }
  style controlW = { size({ w: 64, max-w: full }) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style exampleLead = { fg(muted-foreground).type({ size: sm }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

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
      <doc-example
        title="Group with legend"
        lead="Put a legend first, then related controls. Legend is documented here — there is no separate hub page for it."
        code={exGroup()}
      >
        <fieldset style(controlW)>
          <legend>Account details</legend>
          <column gap="2">
            <input placeholder="Name" />
            <input placeholder="Email" />
          </column>
        </fieldset>
      </doc-example>
      <doc-example
        title="Disabled set"
        lead="disabled freezes every control inside; the legend still names the group."
        code={exDisabled()}
      >
        <fieldset disabled style(controlW)>
          <legend>Billing (locked)</legend>
          <column gap="2">
            <input placeholder="Card" />
            <button type="button" variant="solid" size="sm">Save</button>
          </column>
        </fieldset>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/field">Field</a>
  </row>
</column>
