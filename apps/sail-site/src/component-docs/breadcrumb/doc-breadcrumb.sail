<!--
  <doc-breadcrumb> — /components/breadcrumb
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Trail host only — no host props; compose nested a / small.
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style mutedXs = { fg(muted-foreground).type({ size: xs }) }
  style specimen = { size({ w: full, max-w: panel-max-w-md }).pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-breadcrumb.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const size = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SIZES.length - 1)
    return SIZES[i] ?? 'md'
  })

  const entry = DETAILS.breadcrumb
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const sz = size() !== 'md' ? ` size="${size()}"` : ''
    return `<breadcrumb${sz}>\n  <a href="/components">components</a>\n  <a href="/docs">docs</a>\n  <small>breadcrumb</small>\n</breadcrumb>`
  })

  const bcExPath = () =>
    `<breadcrumb>\n  <a href="/">Home</a>\n  <a href="/components">Components</a>\n  <small>breadcrumb</small>\n</breadcrumb>`
  const bcExDeep = () =>
    `<breadcrumb>\n  <a href="/docs">Docs</a>\n  <a href="/docs/reference">Reference</a>\n  <small>API</small>\n</breadcrumb>`

  return {
    sizeIdx,
    size,
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
      <breadcrumb size={size()}>
        <a href="/components">components</a>
        <a href="/docs">docs</a>
        <small>breadcrumb</small>
      </breadcrumb>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{size()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size" ticks="sm,md,lg"></slider>
        <p style(mutedXs)>
          Type scale on the trail host. Compose nested <code>a</code> crumbs and an inert
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
