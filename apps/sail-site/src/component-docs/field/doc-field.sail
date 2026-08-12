<!--
  <doc-field> — /components/field
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

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

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(64).maxW(full).pad(2).overflow(visible) }
  style controlW = { box: w(64).maxW(full) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
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
        control too for the destructive border.
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
      <column gap="3" style(exampleSection)>
        <h3 style(exampleH)>Label · control · hint</h3>
        <p style(exampleLead)>
          Slot the label, control, and optional hint. The field spaces the stack.
        </p>
        <field style(controlW)>
          <label slot="label">Email</label>
          <input type="email" placeholder="you@ship.dev" />
          <span slot="hint">We will never share this.</span>
          <span slot="error">Enter a valid email.</span>
        </field>
        <doc-code code={exSlots()} lang="sail"></doc-code>
      </column>
      <column gap="3" style(exampleSection)>
        <h3 style(exampleH)>Invalid</h3>
        <p style(exampleLead)>
          Host <code>invalid</code> hides the hint and reveals the error.
        </p>
        <field invalid style(controlW)>
          <label slot="label">Email</label>
          <input type="email" value="not-an-email" invalid />
          <span slot="hint">We will never share this.</span>
          <span slot="error">Enter a valid email.</span>
        </field>
        <doc-code code={exInvalid()} lang="sail"></doc-code>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/components/label">Label</a>
  </row>
</column>
