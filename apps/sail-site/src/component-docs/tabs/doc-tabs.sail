<!--
  <doc-tabs> — /components/tabs
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: active · orientation; onTabChange live.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style specimen = { box: w(full).maxW(panel-max-w-md).pad(2).overflow(visible) }
  style fullW = { box: w(full) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
</scope>

<controller>
import type { Scope } from './doc-tabs.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const active = signal('overview')
  const orientation = signal('horizontal')
  const onTabChange = (detail: { name?: string }): void => {
    const n = detail?.name ?? ''
    active.set(n)
  }

  const entry = DETAILS.tabs
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`active="${active()}"`]
    if (orientation() !== 'horizontal') a.push(`orientation="${orientation()}"`)
    a.push('on-tab-change={onTabChange}')
    return `<tabs ${a.join(' ')}>\n  <tab-list variant="line">\n    <tab name="overview">Overview</tab>\n    <tab name="activity">Activity</tab>\n    <tab name="settings">Settings</tab>\n  </tab-list>\n  <tab-panel name="overview">…</tab-panel>\n  <tab-panel name="activity">…</tab-panel>\n  <tab-panel name="settings">…</tab-panel>\n</tabs>`
  })

  const tabsExLine = () =>
    `<tabs active="overview" orientation="horizontal" on-tab-change={onTabChange}>\n  <tab-list variant="line">\n    <tab name="overview">Overview</tab>\n    <tab name="activity">Activity</tab>\n  </tab-list>\n  <tab-panel name="overview">…</tab-panel>\n  <tab-panel name="activity">…</tab-panel>\n</tabs>`
  const tabsExPill = () =>
    `<tabs active="a">\n  <tab-list variant="pill">\n    <tab name="a">A</tab>\n    <tab name="b">B</tab>\n  </tab-list>\n  <tab-panel name="a">…</tab-panel>\n  <tab-panel name="b">…</tab-panel>\n</tabs>`
  const tabsExVertical = () =>
    `<tabs active="one" orientation="vertical">\n  <tab-list variant="line">\n    <tab name="one">One</tab>\n    <tab name="two">Two</tab>\n  </tab-list>\n  <tab-panel name="one">…</tab-panel>\n  <tab-panel name="two">…</tab-panel>\n</tabs>`

  return {
    active,
    orientation,
    onTabChange,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    tabsExLine,
    tabsExPill,
    tabsExVertical,
  }
})
</controller>

<column gap="6" style(page)>
  <sail-component-viewer
    name={heading()}
    blurb={blurb()}
    pack="Component primitives"
    layout="split"
    snippet={specimenCode()}
    usage-live
    usage={usageCode()}
    rows={props()}
  >
    <column slot="specimen" gap="3" align="stretch" style(specimen)>
      <tabs active={active()} orientation={orientation()} on-tab-change={onTabChange} style(fullW)>
        <tab-list variant="line">
          <tab name="overview">Overview</tab>
          <tab name="activity">Activity</tab>
          <tab name="settings">Settings</tab>
        </tab-list>
        <tab-panel name="overview"><p style(mutedSm)>Overview panel content.</p></tab-panel>
        <tab-panel name="activity"><p style(mutedSm)>Activity panel content.</p></tab-panel>
        <tab-panel name="settings"><p style(mutedSm)>Settings panel content.</p></tab-panel>
      </tabs>
    </column>

    <column slot="knobs" gap="4" align="stretch">
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
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <tabs active="overview" style(fullW)>
        <tab-list variant="line">
          <tab name="overview">Overview</tab>
          <tab name="activity">Activity</tab>
        </tab-list>
        <tab-panel name="overview"><p style(mutedSm)>Overview.</p></tab-panel>
        <tab-panel name="activity"><p style(mutedSm)>Activity.</p></tab-panel>
      </tabs>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Line strip"
        lead={'Default strip look via tab-list variant="line" — not a host prop.'}
        code={tabsExLine()}
      >
        <tabs active="overview">
          <tab-list variant="line">
            <tab name="overview">Overview</tab>
            <tab name="activity">Activity</tab>
          </tab-list>
          <tab-panel name="overview"><p style(mutedSm)>Overview.</p></tab-panel>
          <tab-panel name="activity"><p style(mutedSm)>Activity.</p></tab-panel>
        </tabs>
      </doc-example>
      <doc-example
        title="Pill strip"
        lead={'variant="pill" on the list for a rail look.'}
        code={tabsExPill()}
      >
        <tabs active="a">
          <tab-list variant="pill">
            <tab name="a">A</tab>
            <tab name="b">B</tab>
          </tab-list>
          <tab-panel name="a"><p style(mutedSm)>A panel.</p></tab-panel>
          <tab-panel name="b"><p style(mutedSm)>B panel.</p></tab-panel>
        </tabs>
      </doc-example>
      <doc-example
        title="Vertical"
        lead={'orientation="vertical" stacks the list beside panels.'}
        code={tabsExVertical()}
      >
        <tabs active="one" orientation="vertical">
          <tab-list variant="line">
            <tab name="one">One</tab>
            <tab name="two">Two</tab>
          </tab-list>
          <tab-panel name="one"><p style(mutedSm)>One.</p></tab-panel>
          <tab-panel name="two"><p style(mutedSm)>Two.</p></tab-panel>
        </tabs>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
