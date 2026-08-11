<!--
  <doc-tooltip> — first-class docs host for /components/tooltip.
  Specimen left, knobs right — placement · content.
-->

<controller>
import type { Scope } from './doc-tooltip.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placement = signal('top')
  const content = signal('Copy URL')
  const tipMeta = derived(() => `${placement()} · "${content()}" · component.tooltip.bg/fg`)
  const entry = DETAILS.tooltip
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const tipExTop = () =>
    `<tooltip content="Copy URL" placement="top">\n  <span>ship.dev/d/ab12</span>\n</tooltip>`
  const tipExSides = () =>
    `<tooltip content="Top" placement="top">…</tooltip>\n<tooltip content="Bottom" placement="bottom">…</tooltip>`
  const tipExDanger = () =>
    `<tooltip content="Deletes the branch — cannot be undone" placement="top">\n  <button tone="destructive" variant="outline">Delete branch</button>\n</tooltip>`
  return {
    placement, content, tipMeta,
    heading, blurb, props, usageCode, tipExTop, tipExSides, tipExDanger,
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
            <tooltip content={content()} placement={placement()}>
              <span style(mutedSm)>ship.dev/d/ab12</span>
            </tooltip>
            <p style(monoXs)>{tipMeta()}</p>
            <p style(mutedXs)>Hover or focus the label to open the bubble.</p>
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
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Content</label>
            <select size="sm" bind:value={content} label="Content">
              <option value="Copy URL">Copy URL</option>
              <option value="Open in new tab">Open in new tab</option>
              <option value="Deletes the branch — cannot be undone">Destructive hint</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.tooltip.bg</code> · <code>fg</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Default top</h3>
        <p style(exampleLead)>content attribute fills the bubble; never steals focus.</p>
        <tooltip content="Copy URL" placement="top">
          <span style(mutedSm)>ship.dev/d/ab12</span>
        </tooltip>
        <code-block code={tipExTop()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Four sides</h3>
        <p style(exampleLead)>placement top · bottom · left · right.</p>
        <row gap="4" align="center">
          <tooltip content="Top" placement="top"><button size="sm" variant="outline">top</button></tooltip>
          <tooltip content="Bottom" placement="bottom"><button size="sm" variant="outline">bottom</button></tooltip>
          <tooltip content="Left" placement="left"><button size="sm" variant="outline">left</button></tooltip>
          <tooltip content="Right" placement="right"><button size="sm" variant="outline">right</button></tooltip>
        </row>
        <code-block code={tipExSides()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Destructive hint</h3>
        <p style(exampleLead)>Longer content string on a danger control.</p>
        <tooltip content="Deletes the branch — cannot be undone" placement="top">
          <button tone="destructive" variant="outline" size="sm">Delete branch</button>
        </tooltip>
        <code-block code={tipExDanger()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
