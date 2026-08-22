<!--
  <doc-field> — /components/field
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 64, max-w: 100% }).pad(2).overflow({ all: visible }) }
  style controlW = { size({ w: 64, max-w: 100% }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style exampleLead = { fg(muted-foreground).type({ size: sm }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-field.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const fieldInvalid = signal(false)
  const email = signal('ada@ship.dev')

  const entry = DETAILS.field
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const inv = fieldInvalid() ? ' invalid' : ''
    return `<field${inv}>\n  <label slot="label">Email</label>\n  <input type="email" placeholder="you@ship.dev" />\n  <span slot="hint">We will never share this.</span>\n  <span slot="error">Enter a valid email.</span>\n</field>`
  })

  const exSlots = () =>
    `<field>\n  <label slot="label">Email</label>\n  <input type="email" placeholder="you@ship.dev" />\n  <span slot="hint">We will never share this.</span>\n  <span slot="error">Enter a valid email.</span>\n</field>`
  const exInvalid = () =>
    `<field invalid>\n  <label slot="label">Email</label>\n  <input type="email" value="not-an-email" invalid />\n  <span slot="hint">We will never share this.</span>\n  <span slot="error">Enter a valid email.</span>\n</field>`

  return {
    fieldInvalid,
    email,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    exSlots,
    exInvalid,
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
      <if {fieldInvalid()}>
        <field invalid style(controlW)>
          <label slot="label">Email</label>
          <input type="email" placeholder="you@ship.dev" bind:value={email} invalid />
          <span slot="hint">We will never share this.</span>
          <span slot="error">Enter a valid email address.</span>
        </field>
      {else =>
        <field style(controlW)>
          <label slot="label">Email</label>
          <input type="email" placeholder="you@ship.dev" bind:value={email} />
          <span slot="hint">We will never share this.</span>
          <span slot="error">Enter a valid email address.</span>
        </field>}</if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <row gap="4" align="center">
        <label><checkbox bind:checked={fieldInvalid} size="sm" /> Invalid</label>
      </row>
      <p style(exampleLead)>
        When invalid, the hint hides and the error shows. Put <code>invalid</code> on the
        control too for the danger border.
      </p>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(specimen)>
      <field style(controlW)>
        <label slot="label">Email</label>
        <input type="email" placeholder="you@ship.dev" />
        <span slot="hint">We will never share this.</span>
        <span slot="error">Enter a valid email.</span>
      </field>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Label · control · hint"
        lead="Slot the label, control, and optional hint. The field spaces the stack."
        code={exSlots()}
      >
        <field style(controlW)>
          <label slot="label">Email</label>
          <input type="email" placeholder="you@ship.dev" />
          <span slot="hint">We will never share this.</span>
          <span slot="error">Enter a valid email.</span>
        </field>
      </doc-example>
      <doc-example
        title="Invalid"
        lead="Set invalid to hide the hint and reveal the error."
        code={exInvalid()}
      >
        <field invalid style(controlW)>
          <label slot="label">Email</label>
          <input type="email" value="not-an-email" invalid />
          <span slot="hint">We will never share this.</span>
          <span slot="error">Enter a valid email.</span>
        </field>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/label">Label</a>
  </row>
</column>
