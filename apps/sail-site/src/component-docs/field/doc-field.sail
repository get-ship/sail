<!--
  <doc-field> — first-class docs host for /components/field.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-field.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const fieldInvalid = signal(false)
  const email = signal('ada@ship.dev')
  const fieldMeta = derived(() => {
    return `invalid ${fieldInvalid() ? 'on' : 'off'} · hint/error component.field.* · describedControl`
  })
  const entry = DETAILS.field
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const fldExSlots = () =>
    `<field>\n  <label slot="label">Email</label>\n  <input type="email" placeholder="you@ship.dev" />\n  <span slot="hint">We will never share this.</span>\n  <span slot="error">Enter a valid email.</span>\n</field>`
  const fldExInvalid = () =>
    `<field invalid>\n  <label slot="label">Email</label>\n  <input type="email" value="not-an-email" invalid />\n  <span slot="hint">We will never share this.</span>\n  <span slot="error">Enter a valid email.</span>\n</field>\n<!-- invalid hides hint, shows error + aria-invalid -->`
  const fldExRest = () =>
    `<field>\n  <label slot="label">Name</label>\n  <input placeholder="Ada Lovelace" />\n  <span slot="hint">Shown when valid.</span>\n</field>\n<!-- hint → component.field.hint; control surface component.field.* -->`
  return {
    fieldInvalid, email, fieldMeta,
    heading, blurb, props, usageCode,
    fldExSlots, fldExInvalid, fldExRest,
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
            <if {fieldInvalid()}>
              <field invalid style(stageCol)>
                <label slot="label">Email</label>
                <input type="email" placeholder="you@ship.dev" bind:value={email} invalid />
                <span slot="hint">We will never share this.</span>
                <span slot="error">Enter a valid email address.</span>
              </field>
            {else =>
              <field style(stageCol)>
                <label slot="label">Email</label>
                <input type="email" placeholder="you@ship.dev" bind:value={email} />
                <span slot="hint">We will never share this.</span>
                <span slot="error">Enter a valid email address.</span>
              </field>}</if>
            <p style(monoXs)>{fieldMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Invalid</label>
            <row gap="2" align="center">
              <switch bind:checked={fieldInvalid} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{fieldInvalid() ? 'error shown' : 'hint shown'}</span>
            </row>
            <p style(mutedXs)>
              Host <code>invalid</code> swaps hint → error via
              <code>group-invalid</code>. <code>use:describedControl</code> stamps
              <code>aria-describedby</code> / <code>aria-invalid</code> on the control.
              Put <code>invalid</code> on the input too for the destructive border.
            </p>
          </column>
          <column gap="2">
            <p style(mutedXs)>
              Hint ink <code>component.field.hint</code>; error
              <code>component.field.error</code>. Control chrome is the shared
              <code>component.field.*</code> surface (bg / border / fg).
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Label · control · hint</h3>
        <p style(exampleLead)>
          Slot the label, control, and optional hint. The field spaces the stack and
          wires describedby when both regions exist.
        </p>
        <field style(stageCol)>
          <label slot="label">Email</label>
          <input type="email" placeholder="you@ship.dev" />
          <span slot="hint">We will never share this.</span>
          <span slot="error">Enter a valid email.</span>
        </field>
        <code-block code={fldExSlots()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Invalid</h3>
        <p style(exampleLead)>
          Host <code>invalid</code> hides the hint and reveals the error alert.
          Mirror <code>invalid</code> on the control for the border.
        </p>
        <field invalid style(stageCol)>
          <label slot="label">Email</label>
          <input type="email" value="not-an-email" invalid />
          <span slot="hint">We will never share this.</span>
          <span slot="error">Enter a valid email.</span>
        </field>
        <code-block code={fldExInvalid()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Hint/error leaves and the shared control surface are themeable —
          no filled/unstyled host invent.
        </p>
        <field style(stageCol)>
          <label slot="label">Name</label>
          <input placeholder="Ada Lovelace" />
          <span slot="hint">Shown when valid.</span>
          <span slot="error">Required.</span>
        </field>
        <code-block code={fldExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
