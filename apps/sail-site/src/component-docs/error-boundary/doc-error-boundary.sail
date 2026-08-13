<!--
  <doc-error-boundary> — /components/error-boundary
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style specimen = { box: size({ w: full, max-w: panel-max-w-md }).space({ pad: 2 }).overflow({ all: visible }) }
  style fullW = { box: size({ w: full }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style mutedXs = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style centerBox = { box: space({ pad: 4 }), paint: border({ color: border-subtle, width: 1 }).radius(md) }
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
import type { Scope } from './doc-error-boundary.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const message = signal('This panel could not load.')

  const entry = DETAILS['error-boundary']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const msg = message() || 'This panel could not load.'
    return `<error-boundary message="${msg}">\n  …\n</error-boundary>`
  })

  const ebExWrap = () =>
    `<error-boundary message="This panel could not load.">\n  …\n</error-boundary>`

  return {
    message,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    ebExWrap,
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
      <div style(centerBox)>
        <error-boundary message={message()}>
          <p style(mutedSm)>Healthy child tree — the boundary stays invisible.</p>
        </error-boundary>
      </div>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Message</label>
        <input
          type="text"
          bind:value={message}
          size="sm"
          placeholder="fallback copy"
          style(fullW)
       />
        <p style(mutedXs)>User-facing fallback copy — not the raw Error string.</p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <error-boundary message="This panel could not load.">
        <p style(mutedSm)>Healthy content — boundary invisible.</p>
      </error-boundary>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Wrap a risky subtree"
        lead="A throw inside swaps to the built-in fallback. Retry restores content."
        code={ebExWrap()}
      >
        <error-boundary message="This panel could not load.">
          <p style(mutedSm)>Healthy content — boundary invisible.</p>
        </error-boundary>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
