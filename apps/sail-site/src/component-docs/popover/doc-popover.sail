<!--
  <doc-popover> — first-class docs host for /components/popover.
  placement string; real popovertarget trigger + static surface notes.
-->

<controller>
import type { Scope } from './doc-popover.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placement = signal('bottom')
  const popMeta = derived(() => `placement "${placement()}" · component.popover.bg/fg/border`)
  const entry = DETAILS.popover
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const popExBasic = () =>
    `<button popovertarget="pop">Open popover</button>\n<popover id="pop" placement="bottom">\n  Panel content\n</popover>`
  const popExFilter = () =>
    `<button popovertarget="filters">Filters</button>\n<popover id="filters" placement="bottom-start">…</popover>`
  const popExFaces = () =>
    `<!-- paint → component.popover.bg · fg · border -->\n<popover id="p" placement="top">…</popover>`
  return {
    placement, popMeta,
    heading, blurb, props, usageCode, popExBasic, popExFilter, popExFaces,
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
            <button size="sm" variant="outline" popovertarget="doc-pop-live">Open popover</button>
            <popover id="doc-pop-live" placement={placement()}>
              <column gap="2">
                <p style(mutedSm)>Anchored panel — Escape and light-dismiss from the platform.</p>
              </column>
            </popover>
            <p style(monoXs)>{popMeta()}</p>
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
              <option value="bottom-start">bottom-start</option>
              <option value="top-end">top-end</option>
            </select>
            <p style(mutedXs)>Free string on the host — common anchors listed for knobs.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.popover.bg / fg / border</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Basic anchor</h3>
        <p style(exampleLead)><code>popovertarget</code> + <code>id</code> — no refs.</p>
        <button size="sm" variant="outline" popovertarget="doc-pop-ex1">Open</button>
        <popover id="doc-pop-ex1" placement="bottom">
          <p style(mutedSm)>Panel content</p>
        </popover>
        <code-block code={popExBasic()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Filters</h3>
        <p style(exampleLead)>bottom-start hugs a toolbar control.</p>
        <code-block code={popExFilter()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Faces</h3>
        <p style(exampleLead)>Theme panel leaves in Studio.</p>
        <code-block code={popExFaces()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
