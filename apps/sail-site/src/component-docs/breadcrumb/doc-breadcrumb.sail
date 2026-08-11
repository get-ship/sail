<!--
  <doc-breadcrumb> — first-class docs host for /components/breadcrumb.
  Trail host only — no PE props; compose nested a / small.
-->

<controller>
import type { Scope } from './doc-breadcrumb.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const entry = DETAILS.breadcrumb
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const bcMeta = () => 'trail host · component.breadcrumb.fg / current · separators via compiler'
  const bcExPath = () =>
    `<breadcrumb>\n  <a href="/">Home</a>\n  <a href="/components">Components</a>\n  <small>breadcrumb</small>\n</breadcrumb>`
  const bcExDeep = () =>
    `<breadcrumb>\n  <a href="/docs">Docs</a>\n  <a href="/docs/reference">Reference</a>\n  <small>API</small>\n</breadcrumb>`
  const bcExFaces = () =>
    `<!-- faces: component.breadcrumb.fg (trail) · component.breadcrumb.current -->\n<breadcrumb>\n  <a href="/">Home</a>\n  <small>Here</small>\n</breadcrumb>`
  return {
    heading, blurb, props, usageCode, bcMeta, bcExPath, bcExDeep, bcExFaces,
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
  style swatchRow = { box: w(full) }
  style swatch = {
    box: size(6).shrink(0),
    paint: radius(full).borderWidth(1).border(border).cursor(pointer),
  }
  style swatchRest = { paint: bg(surface-1) }
  style swatchPrimary = { paint: bg(primary) }
  style swatchInfo = { paint: bg(info) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted) }
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
          <column gap="3" align="center" style(stageCol)>
            <breadcrumb>
              <a href="/components">components</a>
              <a href="/docs">docs</a>
              <small>breadcrumb</small>
            </breadcrumb>
            <p style(monoXs)>{bcMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Props</label>
            <p style(mutedXs)>
              No PE props on the trail host. Compose nested <code>a</code> crumbs and an inert
              current (commonly <code>small</code>).
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.breadcrumb.fg</code> — trail / separators</p>
              <p style(mutedXs)><code>component.breadcrumb.current</code> — current page ink</p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Path trail</h3>
        <p style(exampleLead)>Anchors for the path; <code>small</code> for where you are.</p>
        <breadcrumb>
          <a href="/">Home</a>
          <a href="/components">Components</a>
          <small>breadcrumb</small>
        </breadcrumb>
        <code-block code={bcExPath()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Deep crumbs</h3>
        <p style(exampleLead)>Any depth — separators draw before every non-first crumb.</p>
        <breadcrumb>
          <a href="/docs">Docs</a>
          <a href="/docs/reference">Reference</a>
          <small>API</small>
        </breadcrumb>
        <code-block code={bcExDeep()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Faces</h3>
        <p style(exampleLead)>Theme <code>component.breadcrumb.fg</code> and <code>current</code> in Studio.</p>
        <code-block code={bcExFaces()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
