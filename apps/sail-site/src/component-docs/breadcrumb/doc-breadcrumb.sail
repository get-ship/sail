<!--
  <doc-breadcrumb> — /components/breadcrumb
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Trail host only — no host props; compose nested a / small.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style mutedXs = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style specimen = { box: size({ w: full, max-w: panel-max-w-md }).space({ pad: 2 }).overflow({ all: visible }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
</scope>

<controller>
import type { Scope } from './doc-breadcrumb.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const entry = DETAILS.breadcrumb
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = () =>
    `<breadcrumb>\n  <a href="/components">components</a>\n  <a href="/docs">docs</a>\n  <small>breadcrumb</small>\n</breadcrumb>`

  const bcExPath = () =>
    `<breadcrumb>\n  <a href="/">Home</a>\n  <a href="/components">Components</a>\n  <small>breadcrumb</small>\n</breadcrumb>`
  const bcExDeep = () =>
    `<breadcrumb>\n  <a href="/docs">Docs</a>\n  <a href="/docs/reference">Reference</a>\n  <small>API</small>\n</breadcrumb>`

  return {
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    bcExPath,
    bcExDeep,
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
    <column slot="specimen" gap="3" align="center" style(specimen)>
      <breadcrumb>
        <a href="/components">components</a>
        <a href="/docs">docs</a>
        <small>breadcrumb</small>
      </breadcrumb>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Composition</label>
        <p style(mutedXs)>
          No host props on the trail host. Compose nested <code>a</code> crumbs and an inert
          current (commonly <code>small</code>).
        </p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="start" style(specimen)>
      <breadcrumb>
        <a href="/">Home</a>
        <a href="/components">Components</a>
        <small>breadcrumb</small>
      </breadcrumb>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Path trail"
        lead="Anchors for the path; small for where you are."
        code={bcExPath()}
      >
        <breadcrumb>
          <a href="/">Home</a>
          <a href="/components">Components</a>
          <small>breadcrumb</small>
        </breadcrumb>
      </doc-example>
      <doc-example
        title="Deep crumbs"
        lead="Any depth — separators draw before every non-first crumb."
        code={bcExDeep()}
      >
        <breadcrumb>
          <a href="/docs">Docs</a>
          <a href="/docs/reference">Reference</a>
          <small>API</small>
        </breadcrumb>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
