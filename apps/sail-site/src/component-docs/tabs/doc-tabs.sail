<!--
  <doc-tabs> — /components/tabs
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: orientation. Tabs themselves pick the active panel.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style mutedSm = { fg(muted-foreground).type({ size: sm }) }
  style mutedXs = { fg(muted-foreground).type({ size: xs }) }
  style specimen = { size({ w: 100%, max-w: panel-max-w-md }).pad(2).overflow({ all: visible }) }
  style fullW = { size({ w: 100% }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-tabs.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const orientation = signal('horizontal')

  const entry = DETAILS.tabs
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (orientation() !== 'horizontal') a.push(`orientation="${orientation()}"`)
    const open = a.length ? `<tabs ${a.join(' ')}>` : `<tabs>`
    return `${open}\n  <tab title="Overview">…</tab>\n  <tab title="Activity">…</tab>\n  <tab title="Settings">…</tab>\n</tabs>`
  })

  const tabsExLine = () =>
    `<tabs>\n  <tab title="Overview">…</tab>\n  <tab title="Activity">…</tab>\n</tabs>`
  const tabsExPill = () =>
    `<tabs>\n  <tab title="A" variant="pill">…</tab>\n  <tab title="B" variant="pill">…</tab>\n</tabs>`
  const tabsExVertical = () =>
    `<tabs orientation="vertical">\n  <tab title="One">…</tab>\n  <tab title="Two">…</tab>\n</tabs>`

  return {
    orientation,
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
      <tabs orientation={orientation()} style(fullW)>
        <tab title="Overview"><p style(mutedSm)>Overview panel content.</p></tab>
        <tab title="Activity"><p style(mutedSm)>Activity panel content.</p></tab>
        <tab title="Settings"><p style(mutedSm)>Settings panel content.</p></tab>
      </tabs>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Orientation</label>
        <select size="sm" bind:value={orientation} label="Orientation">
          <option value="horizontal">horizontal</option>
          <option value="vertical">vertical</option>
        </select>
        <p style(mutedXs)>Click a tab to change the panel. First tab is selected by default.</p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <tabs style(fullW)>
        <tab title="Overview"><p style(mutedSm)>Overview.</p></tab>
        <tab title="Activity"><p style(mutedSm)>Activity.</p></tab>
      </tabs>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Line strip"
        lead={'Default strip look via tab-list variant="line" — not a host prop.'}
        code={tabsExLine()}
      >
        <tabs>
          <tab title="Overview"><p style(mutedSm)>Overview.</p></tab>
          <tab title="Activity"><p style(mutedSm)>Activity.</p></tab>
        </tabs>
      </doc-example>
      <doc-example
        title="Pill strip"
        lead={'variant="pill" on the list for a rail look.'}
        code={tabsExPill()}
      >
        <tabs>
          <tab title="A" variant="pill"><p style(mutedSm)>A panel.</p></tab>
          <tab title="B" variant="pill"><p style(mutedSm)>B panel.</p></tab>
        </tabs>
      </doc-example>
      <doc-example
        title="Vertical"
        lead={'orientation="vertical" puts the list in a column beside the panels.'}
        code={tabsExVertical()}
      >
        <tabs orientation="vertical">
          <tab title="One"><p style(mutedSm)>One.</p></tab>
          <tab title="Two"><p style(mutedSm)>Two.</p></tab>
        </tabs>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
