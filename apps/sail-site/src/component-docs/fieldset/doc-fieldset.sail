<!--
  <doc-fieldset> — first-class docs host for /components/fieldset.
  Owns <legend> documentation (no top-level legend hub tile).
-->

<controller>
import type { Scope } from './doc-fieldset.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const fieldsetDisabled = signal(false)
  const fieldsetMeta = derived(() => {
    return `disabled ${fieldsetDisabled() ? 'on' : 'off'} · border component.fieldset.border · legend caption`
  })

  const entry = DETAILS.fieldset
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const fsExGroup = () =>
    `<fieldset>\n  <legend>Account details</legend>\n  <column gap="2">\n    <input placeholder="Name" />\n    <input placeholder="Email" />\n  </column>\n</fieldset>`
  const fsExDisabled = () =>
    `<fieldset disabled>\n  <legend>Billing (locked)</legend>\n  <column gap="2">\n    <input placeholder="Card" />\n    <button type="button" variant="solid" size="sm">Save</button>\n  </column>\n</fieldset>\n<!-- native disabled freezes controls; PE dims opacity -->`
  const fsExRest = () =>
    `<fieldset>\n  <legend>Profile</legend>\n  …\n</fieldset>\n<!-- border → component.fieldset.border; legend ink → component.legend.fg -->`
  const legendExLeaf = () =>
    `<fieldset>\n  <legend>Themeable caption</legend>\n  <!-- ink from component.legend.fg (--legend-foreground) -->\n</fieldset>`

  return {
    fieldsetDisabled,
    fieldsetMeta,
    heading,
    blurb,
    props,
    usageCode,
    fsExGroup,
    fsExDisabled,
    fsExRest,
    legendExLeaf,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageColMd = { box: w(panel-max-w-md).maxW(full) }
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
            <if {fieldsetDisabled()}>
              <fieldset disabled style(stageCol)>
                <legend>Account details</legend>
                <column gap="2">
                  <input placeholder="Name" />
                  <input placeholder="Email" />
                  <button type="button" variant="solid" size="sm">Save</button>
                </column>
              </fieldset>
            {else =>
              <fieldset style(stageCol)>
                <legend>Account details</legend>
                <column gap="2">
                  <input placeholder="Name" />
                  <input placeholder="Email" />
                  <button type="button" variant="solid" size="sm">Save</button>
                </column>
              </fieldset>}</if>
            <p style(monoXs)>{fieldsetMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Disabled</label>
            <row gap="2" align="center">
              <switch bind:checked={fieldsetDisabled} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{fieldsetDisabled() ? 'on' : 'off'}</span>
            </row>
            <p style(mutedXs)>
              Native <code>disabled</code> freezes every control inside the group.
              Style PE dims opacity and sets <code>cursor: not-allowed</code>.
              Border always paints <code>component.fieldset.border</code>.
              Caption is a nested <code>&lt;legend&gt;</code> (documented below — not a hub tile).
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Group with legend</h3>
        <p style(exampleLead)>
          Put a native <code>legend</code> first, then related controls. Form grouping
          stays platform — fieldset owns the border; legend paints
          <code>component.legend.fg</code>. There is no top-level legend docs page.
        </p>
        <fieldset style(stageCol)>
          <legend>Account details</legend>
          <column gap="2">
            <input placeholder="Name" />
            <input placeholder="Email" />
          </column>
        </fieldset>
        <code-block code={fsExGroup()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Disabled set</h3>
        <p style(exampleLead)>
          <code>disabled</code> is the only PE prop. The platform freezes every control
          inside; Style dims the host. The legend still names the group.
        </p>
        <fieldset disabled style(stageCol)>
          <legend>Billing (locked)</legend>
          <column gap="2">
            <input placeholder="Card" />
            <button type="button" variant="solid" size="sm">Save</button>
          </column>
        </fieldset>
        <code-block code={fsExDisabled()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest border (themeable)</h3>
        <p style(exampleLead)>
          Chrome border always rides <code>component.fieldset.border</code>
          (<code>--fieldset-border</code>). Override that leaf in Studio themes — there is
          no tone or variant axis.
        </p>
        <fieldset style(stageCol)>
          <legend>Profile</legend>
          <column gap="2">
            <input placeholder="Display name" />
          </column>
        </fieldset>
        <code-block code={fsExRest()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Legend leaf</h3>
        <p style(exampleLead)>
          Caption ink is always <code>component.legend.fg</code>
          (<code>--legend-foreground</code>). No size, variant, or tone props on
          <code>&lt;legend&gt;</code> — nest it inside fieldset only.
        </p>
        <fieldset style(stageColMd)>
          <legend>Themeable caption</legend>
          <p style(mutedSm)>Caption rides the legend leaf; border is fieldset.</p>
        </fieldset>
        <code-block code={legendExLeaf()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
