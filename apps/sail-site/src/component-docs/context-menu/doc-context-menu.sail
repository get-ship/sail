<!--
  <doc-context-menu> — first-class docs host for /components/context-menu.
  for= target wiring; onSelect live. Static menu surface + right-click demo.
-->

<controller>
import type { Scope } from './doc-context-menu.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const last = signal('')
  const onSelect = (detail: { value?: string }): void => {
    last.set(detail?.value ?? '')
  }
  const cmMeta = derived(() => {
    const l = last() ? ` · selected ${last()}` : ''
    return `for=doc-cm-target${l} · component.context-menu.*`
  })
  const entry = DETAILS['context-menu']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const cmExBasic = () =>
    `<p id="canvas">right-click me</p>\n<context-menu for="canvas" on-select={onSelect}>\n  <menu-item value="cut">Cut</menu-item>\n  <menu-item value="copy">Copy</menu-item>\n</context-menu>`
  const cmExMore = () =>
    `<context-menu for="row">\n  <menu-item value="open">Open</menu-item>\n  <menu-item value="del">Delete</menu-item>\n</context-menu>`
  const cmExFaces = () =>
    `<!-- paint → component.context-menu.bg · fg · border -->`
  return {
    last, onSelect, cmMeta,
    heading, blurb, props, usageCode, cmExBasic, cmExMore, cmExFaces,
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
            <p id="doc-cm-target" style(centerBox)>Right-click this card</p>
            <context-menu for="doc-cm-target" id="doc-cm-live" on-select={onSelect}>
              <menu-item value="copy">Copy id</menu-item>
              <menu-item value="delete">Delete</menu-item>
            </context-menu>
            <p style(monoXs)>{cmMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>For</label>
            <p style(mutedXs)>
              <code>for</code> is the id of the right-click target (wired on stage to
              <code>doc-cm-target</code>). Not a free-form knob — specimen owns the link.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.context-menu.bg / fg / border</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Canvas target</h3>
        <p style(exampleLead)>Opens at the pointer on contextmenu of <code>for</code>.</p>
        <p id="doc-cm-ex1" style(centerBox)>right-click me</p>
        <context-menu for="doc-cm-ex1" on-select={onSelect}>
          <menu-item value="cut">Cut</menu-item>
          <menu-item value="copy">Copy</menu-item>
        </context-menu>
        <code-block code={cmExBasic()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Row actions</h3>
        <code-block code={cmExMore()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Faces</h3>
        <code-block code={cmExFaces()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
