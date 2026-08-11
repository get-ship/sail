<!--
  <doc-scroll-area> — first-class docs host for /components/scroll-area.
  Specimen left, knobs right — orientation only.
-->

<controller>
import type { Scope } from './doc-scroll-area.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const orientation = signal('vertical')
  const saMeta = derived(() => {
    return `${orientation()} · thumb component.scroll-area.thumb · constrain with Style box`
  })
  const entry = DETAILS['scroll-area']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const saExVertical = () =>
    `<scroll-area orientation="vertical" style({ box: h(32).w(full).pad(3), paint: borderWidth(1).border(border).radius(md) })>\n  <p>Overflow content…</p>\n</scroll-area>`
  const saExHorizontal = () =>
    `<scroll-area orientation="horizontal" style({ box: w(full).pad(3), paint: borderWidth(1).border(border).radius(md) })>\n  <p style({ text: whitespace(nowrap) })>Wide content that overflows →</p>\n</scroll-area>`
  const saExRest = () =>
    `<scroll-area>\n  <!-- default orientation=vertical · thumb → component.scroll-area.thumb -->\n</scroll-area>`
  return {
    orientation,
    saMeta,
    heading,
    blurb,
    props,
    usageCode,
    saExVertical,
    saExHorizontal,
    saExRest,
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
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style scrollDemoV = {
    box: h(32).w(full).maxW(panel-max-w-sm).pad(3),
    paint: borderWidth(1).border(border-subtle).radius(md).bg(surface-0),
  }
  style scrollDemoH = {
    box: w(full).maxW(panel-max-w-sm).pad(3),
    paint: borderWidth(1).border(border-subtle).radius(md).bg(surface-0),
  }
  style wideBand = { box: w(panel-max-w-lg), text: whitespace(nowrap) }
  style faceRow = { box: w(full) }
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
          <column gap="3" align="stretch" style(stageCol)>
            <if {orientation() === 'horizontal'}>
              <scroll-area orientation="horizontal" style(scrollDemoH)>
                <p style(mutedSm, wideBand)>
                  Wide band — tokens · signals · a11y · hot reload · SSR · layout · overflow →
                </p>
              </scroll-area>
            {else =>
              <if {orientation() === 'both'}>
                <scroll-area orientation="both" style(scrollDemoV)>
                  <p style(mutedSm)>Line 1 — overflow both axes</p>
                  <p style(mutedSm, wideBand)>Line 2 — wide content that needs horizontal scroll as well →</p>
                  <p style(mutedSm)>Line 3</p>
                  <p style(mutedSm)>Line 4</p>
                  <p style(mutedSm)>Line 5</p>
                  <p style(mutedSm)>Line 6</p>
                  <p style(mutedSm)>Line 7</p>
                  <p style(mutedSm)>Line 8</p>
                </scroll-area>
              {else =>
                <scroll-area orientation="vertical" style(scrollDemoV)>
                  <p style(mutedSm)>Line 1 — overflow content</p>
                  <p style(mutedSm)>Line 2</p>
                  <p style(mutedSm)>Line 3</p>
                  <p style(mutedSm)>Line 4</p>
                  <p style(mutedSm)>Line 5</p>
                  <p style(mutedSm)>Line 6</p>
                  <p style(mutedSm)>Line 7</p>
                  <p style(mutedSm)>Line 8</p>
                </scroll-area>}</if>}</if>
            <p style(monoXs)>{saMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Orientation</label>
            <select size="sm" bind:value={orientation} label="Orientation">
              <option value="vertical">vertical</option>
              <option value="horizontal">horizontal</option>
              <option value="both">both</option>
            </select>
            <p style(mutedXs)>
              vertical = overflow-y auto · horizontal = overflow-x auto · both = overflow auto.
              Constrain with Style box so content overflows.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.scroll-area.thumb</code> — rest thumb</p>
              <p style(mutedXs)><code>component.scroll-area.thumb-hover</code> — hover leaf (parked unread in product)</p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Vertical overflow</h3>
        <p style(exampleLead)>
          Default orientation. Fix height so rows spill and the thumb appears.
        </p>
        <scroll-area orientation="vertical" style(scrollDemoV)>
          <p style(mutedSm)>Line 1</p>
          <p style(mutedSm)>Line 2</p>
          <p style(mutedSm)>Line 3</p>
          <p style(mutedSm)>Line 4</p>
          <p style(mutedSm)>Line 5</p>
          <p style(mutedSm)>Line 6</p>
        </scroll-area>
        <code-block code={saExVertical()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Horizontal overflow</h3>
        <p style(exampleLead)>
          <code>orientation="horizontal"</code> — nowrap band wider than the host.
        </p>
        <scroll-area orientation="horizontal" style(scrollDemoH)>
          <p style(mutedSm, wideBand)>Wide content that overflows horizontally →</p>
        </scroll-area>
        <code-block code={saExHorizontal()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens</h3>
        <p style(exampleLead)>
          Thumb from <code>component.scroll-area.thumb</code>. Hover leaf is registry-present
          but unread in product (SPEC-D1).
        </p>
        <code-block code={saExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
