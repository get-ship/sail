<!--
  <doc-navbar> — first-class docs host for /components/navbar.
  Specimen left, knobs right — placement · sticky · bordered · autohide · offset.
-->

<controller>
import type { Scope } from './doc-navbar.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placement = signal('top')
  const sticky = signal(false)
  const bordered = signal(true)
  const autohide = signal(false)
  const offset = signal('0')
  const navMeta = derived(() => {
    const flags = [
      sticky() ? 'sticky' : null,
      bordered() ? 'bordered' : null,
      autohide() ? 'autohide' : null,
    ].filter(Boolean)
    const f = flags.length ? flags.join(' · ') : 'rest'
    return `${placement()} · ${f} · offset ${offset()} · component.navbar.*`
  })
  const entry = DETAILS.navbar
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const navExTop = () =>
    `<navbar placement="top" bordered>\n  <strong slot="brand">Ship</strong>\n  <a href="/docs">Docs</a>\n  <button slot="actions" size="sm" variant="outline">Sign in</button>\n</navbar>`
  const navExRail = () =>
    `<navbar placement="rail" bordered sticky>\n  <strong slot="brand">Ship</strong>\n  <a href="/docs">Docs</a>\n</navbar>`
  const navExBottom = () =>
    `<navbar placement="bottom" sticky bordered>\n  <strong slot="brand">Ship</strong>\n  <button slot="actions" size="sm">Account</button>\n</navbar>`
  return {
    placement, sticky, bordered, autohide, offset, navMeta,
    heading, blurb, props, usageCode, navExTop, navExRail, navExBottom,
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
          <column gap="3" align="stretch" style(fullW)>
            <navbar
              placement={placement()}
              sticky={sticky() ? 'true' : null}
              bordered={bordered() ? 'true' : null}
              autohide={autohide() ? 'true' : null}
              offset={offset()}
              style(fullW)
            >
              <strong slot="brand">Ship</strong>
              <a href="/components">Components</a>
              <a href="/docs">Docs</a>
              <button slot="actions" size="sm" variant="outline">Sign in</button>
            </navbar>
            <p style(monoXs)>{navMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Placement</label>
            <select size="sm" bind:value={placement} label="Placement">
              <option value="top">top</option>
              <option value="bottom">bottom</option>
              <option value="rail">rail</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Sticky</label>
            <row gap="2" align="center">
              <switch bind:checked={sticky} size="sm" tone="primary" label="Sticky"></switch>
              <span style(sizeValue)>{sticky() ? 'pinned' : 'flow'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Bordered</label>
            <row gap="2" align="center">
              <switch bind:checked={bordered} size="sm" tone="primary" label="Bordered"></switch>
              <span style(sizeValue)>{bordered() ? 'edge hairline' : 'flat'}</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Autohide</label>
            <row gap="2" align="center">
              <switch bind:checked={autohide} size="sm" tone="primary" label="Autohide"></switch>
              <span style(sizeValue)>{autohide() ? 'tuck on scroll' : 'always'}</span>
            </row>
            <p style(mutedXs)>With sticky + chromeOffset — operator behavior depth later.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Offset</label>
            <select size="sm" bind:value={offset} label="Offset">
              <option value="0">0</option>
              <option value="8">8</option>
              <option value="16">16</option>
              <option value="48">48</option>
            </select>
            <p style(mutedXs)>Sticky top/left inset (number).</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.navbar.bg</code> · <code>fg</code> · <code>border</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Top bar</h3>
        <p style(exampleLead)>Default placement. Brand · links · actions regions.</p>
        <navbar placement="top" bordered style(fullW)>
          <strong slot="brand">Ship</strong>
          <a href="/docs">Docs</a>
          <button slot="actions" size="sm" variant="outline">Sign in</button>
        </navbar>
        <code-block code={navExTop()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Rail</h3>
        <p style(exampleLead)><code>placement="rail"</code> — side nav geometry.</p>
        <navbar placement="rail" bordered style({ box: minH(40) })>
          <strong slot="brand">Ship</strong>
          <a href="/docs">Docs</a>
          <a href="/components">Components</a>
        </navbar>
        <code-block code={navExRail()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Bottom sticky</h3>
        <p style(exampleLead)>Bottom placement pins with sticky.</p>
        <code-block code={navExBottom()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
