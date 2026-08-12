<!--
  <doc-error-boundary> — /components/error-boundary
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(full).maxW(panel-max-w-md).pad(2).overflow(visible) }
  style fullW = { box: w(full) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }
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
