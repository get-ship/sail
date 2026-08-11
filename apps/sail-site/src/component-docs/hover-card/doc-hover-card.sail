<!--
  <doc-hover-card> — first-class docs host for /components/hover-card.
  placement string; trigger + panel demo.
-->

<controller>
import type { Scope } from './doc-hover-card.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placement = signal('top')
  const hcMeta = derived(() => `placement "${placement()}" · component.hover-card.*`)
  const entry = DETAILS['hover-card']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const hcExBasic = () =>
    `<a href="#" popovertarget="profile">@ada</a>\n<hover-card id="profile" placement="top">\n  <strong>Ada Lovelace</strong>\n  <p>Mathematician · first programmer</p>\n</hover-card>`
  const hcExBottom = () =>
    `<hover-card id="h" placement="bottom">…</hover-card>`
  const hcExFaces = () =>
    `<!-- paint → component.hover-card.bg · fg · border (aliases popover) -->`
  return {
    placement, hcMeta,
    heading, blurb, props, usageCode, hcExBasic, hcExBottom, hcExFaces,
  }
})
</controller>

<scope>

  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageSplit = { box: w(full).minW(0).minH(64) }
  style stageCanvas = { box: grow(1).basis(0).minW(0).minH(64).pad(8) }
  style stageCol = { box: w(panel-max-w-md).maxW(full) }
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
  style faceRow = { box: w(full) }
  style fullW = { box: w(full) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }

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
          <column gap="3" align="center">
            <a href="#" popovertarget="doc-hc-live">@ada</a>
            <hover-card id="doc-hc-live" placement={placement()}>
              <strong>Ada Lovelace</strong>
              <p style(mutedXs)>Mathematician · first programmer</p>
            </hover-card>
            <p style(monoXs)>{hcMeta()}</p>
            <p style(mutedXs)>Hover the trigger — open/close delays avoid flicker.</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Placement</label>
            <select size="sm" bind:value={placement} label="Placement">
              <option value="top">top</option>
              <option value="bottom">bottom</option>
              <option value="left">left</option>
              <option value="right">right</option>
              <option value="bottom-end">bottom-end</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.hover-card.bg / fg / border</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Profile preview</h3>
        <p style(exampleLead)>popovertarget names the hover-card id.</p>
        <a href="#" popovertarget="doc-hc-ex1">@ada</a>
        <hover-card id="doc-hc-ex1" placement="top">
          <strong>Ada Lovelace</strong>
          <p style(mutedXs)>Mathematician · first programmer</p>
        </hover-card>
        <code-block code={hcExBasic()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Bottom placement</h3>
        <code-block code={hcExBottom()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Faces</h3>
        <code-block code={hcExFaces()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
