<!--
  <doc-tabs> — first-class docs host for /components/tabs.
  Specimen left, knobs right — active · orientation (onTabChange live).
-->

<controller>
import type { Scope } from './doc-tabs.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const active = signal('overview')
  const orientation = signal('horizontal')
  const lastTab = signal('')
  const onTabChange = (detail: { name?: string }): void => {
    const n = detail?.name ?? ''
    active.set(n)
    lastTab.set(n)
  }
  const tabsMeta = derived(() => {
    const ch = lastTab() ? ` · last ${lastTab()}` : ''
    return `active "${active()}" · ${orientation()}${ch} · faces tab* / tab-list*`
  })
  const entry = DETAILS.tabs
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const tabsExLine = () =>
    `<tabs active="overview" orientation="horizontal" on-tab-change={onTabChange}>\n  <tab-list variant="line">\n    <tab name="overview">Overview</tab>\n    <tab name="activity">Activity</tab>\n  </tab-list>\n  <tab-panel name="overview">…</tab-panel>\n  <tab-panel name="activity">…</tab-panel>\n</tabs>`
  const tabsExPill = () =>
    `<tabs active="a">\n  <tab-list variant="pill">\n    <tab name="a">A</tab>\n    <tab name="b">B</tab>\n  </tab-list>\n  <tab-panel name="a">…</tab-panel>\n  <tab-panel name="b">…</tab-panel>\n</tabs>`
  const tabsExVertical = () =>
    `<tabs active="one" orientation="vertical">\n  <tab-list variant="line">\n    <tab name="one">One</tab>\n    <tab name="two">Two</tab>\n  </tab-list>\n  <tab-panel name="one">…</tab-panel>\n  <tab-panel name="two">…</tab-panel>\n</tabs>`
  return {
    active, orientation, lastTab, onTabChange, tabsMeta,
    heading, blurb, props, usageCode,
    tabsExLine, tabsExPill, tabsExVertical,
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
          <column gap="3" align="stretch" style(stageCol)>
            <tabs active={active()} orientation={orientation()} on-tab-change={onTabChange} style(stageCol)>
              <tab-list variant="line">
                <tab name="overview">Overview</tab>
                <tab name="activity">Activity</tab>
                <tab name="settings">Settings</tab>
              </tab-list>
              <tab-panel name="overview"><p style(mutedSm)>Overview panel — host paint from component.tab* / tab-list*.</p></tab-panel>
              <tab-panel name="activity"><p style(mutedSm)>Activity panel.</p></tab-panel>
              <tab-panel name="settings"><p style(mutedSm)>Settings panel.</p></tab-panel>
            </tabs>
            <p style(monoXs)>{tabsMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Active</label>
            <select size="sm" bind:value={active} label="Active">
              <option value="overview">overview</option>
              <option value="activity">activity</option>
              <option value="settings">settings</option>
            </select>
            <p style(mutedXs)>Selected tab name. Live clicks fire <code>onTabChange</code>.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Orientation</label>
            <select size="sm" bind:value={orientation} label="Orientation">
              <option value="horizontal">horizontal</option>
              <option value="vertical">vertical</option>
            </select>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.tab.fg / selected-fg / selected-bg / border</code></p>
              <p style(mutedXs)><code>component.tab-list.bg / border / rail</code> (FACE_MERGE → tabs)</p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Line strip</h3>
        <p style(exampleLead)>Default strip look via <code>tab-list variant="line"</code> — not a host prop.</p>
        <tabs active="overview">
          <tab-list variant="line">
            <tab name="overview">Overview</tab>
            <tab name="activity">Activity</tab>
          </tab-list>
          <tab-panel name="overview"><p style(mutedSm)>Overview.</p></tab-panel>
          <tab-panel name="activity"><p style(mutedSm)>Activity.</p></tab-panel>
        </tabs>
        <code-block code={tabsExLine()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Pill strip</h3>
        <p style(exampleLead)><code>variant="pill"</code> on the list — rail from <code>component.tab-list.rail</code>.</p>
        <tabs active="a">
          <tab-list variant="pill">
            <tab name="a">A</tab>
            <tab name="b">B</tab>
          </tab-list>
          <tab-panel name="a"><p style(mutedSm)>A panel.</p></tab-panel>
          <tab-panel name="b"><p style(mutedSm)>B panel.</p></tab-panel>
        </tabs>
        <code-block code={tabsExPill()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Vertical</h3>
        <p style(exampleLead)><code>orientation="vertical"</code> stacks the list beside panels.</p>
        <tabs active="one" orientation="vertical">
          <tab-list variant="line">
            <tab name="one">One</tab>
            <tab name="two">Two</tab>
          </tab-list>
          <tab-panel name="one"><p style(mutedSm)>One.</p></tab-panel>
          <tab-panel name="two"><p style(mutedSm)>Two.</p></tab-panel>
        </tabs>
        <code-block code={tabsExVertical()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
