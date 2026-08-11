<!--
  <doc-dropdown> — first-class docs host for /components/dropdown.
  placement · onSelect (live on stage).
-->

<controller>
import type { Scope } from './doc-dropdown.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placement = signal('bottom-start')
  const last = signal('')
  const onSelect = (detail: { value?: string }): void => {
    last.set(detail?.value ?? '')
  }
  const ddMeta = derived(() => {
    const l = last() ? ` · selected ${last()}` : ''
    return `placement "${placement()}"${l} · component.dropdown.*`
  })
  const entry = DETAILS.dropdown
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const ddExBasic = () =>
    `<button popovertarget="menu">Actions</button>\n<dropdown id="menu" placement="bottom-start" on-select={onSelect}>\n  <menu-item value="edit">Edit</menu-item>\n  <menu-item value="dup">Duplicate</menu-item>\n</dropdown>`
  const ddExDanger = () =>
    `<dropdown id="m" placement="bottom">\n  <menu-item value="del" tone="destructive">Delete</menu-item>\n</dropdown>`
  const ddExPlace = () =>
    `<dropdown id="m" placement="top-end">…</dropdown>`
  return {
    placement, last, onSelect, ddMeta,
    heading, blurb, props, usageCode, ddExBasic, ddExDanger, ddExPlace,
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
            <button size="sm" variant="outline" popovertarget="doc-dd-live">Row actions ▾</button>
            <dropdown id="doc-dd-live" placement={placement()} on-select={onSelect}>
              <menu-item value="edit">Edit</menu-item>
              <menu-item value="duplicate">Duplicate</menu-item>
              <menu-item value="archive">Archive</menu-item>
            </dropdown>
            <p style(monoXs)>{ddMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Placement</label>
            <select size="sm" bind:value={placement} label="Placement">
              <option value="bottom">bottom</option>
              <option value="bottom-start">bottom-start</option>
              <option value="top">top</option>
              <option value="top-end">top-end</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.dropdown.bg / fg / border</code></p>
              <p style(mutedXs)>Row paint on <code>component.menu-item.*</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Actions menu</h3>
        <p style(exampleLead)>Roving arrows, type-ahead, select closes + emits.</p>
        <button size="sm" variant="outline" popovertarget="doc-dd-ex1">Actions ▾</button>
        <dropdown id="doc-dd-ex1" placement="bottom-start" on-select={onSelect}>
          <menu-item value="edit">Edit</menu-item>
          <menu-item value="dup">Duplicate</menu-item>
        </dropdown>
        <code-block code={ddExBasic()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Destructive item</h3>
        <p style(exampleLead)><code>tone</code> on menu-item for danger rows.</p>
        <code-block code={ddExDanger()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Placement string</h3>
        <p style(exampleLead)>Free PE string — common anchors in knobs.</p>
        <code-block code={ddExPlace()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
