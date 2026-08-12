<!--
  <doc-popover> — /components/popover
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: popovertarget invoker + placement knob.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style specimen = { box: pad(2).overflow(visible) }
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
import type { Scope } from './doc-popover.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placement = signal('bottom')

  const entry = DETAILS.popover
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const place = placement() !== 'bottom' ? ` placement="${placement()}"` : ' placement="bottom"'
    return `<button popovertarget="pop">Open popover</button>\n<popover id="pop"${place}>\n  Panel content\n</popover>`
  })

  const popExBasic = () =>
    `<button popovertarget="pop">Open popover</button>\n<popover id="pop" placement="bottom">\n  Panel content\n</popover>`
  const popExFilter = () =>
    `<button popovertarget="filters">Filters</button>\n<popover id="filters" placement="bottom-start">…</popover>`

  return {
    placement,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    popExBasic,
    popExFilter,
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
      <button size="sm" variant="outline" popovertarget="doc-pop-live">Open popover</button>
      <popover id="doc-pop-live" placement={placement()}>
        <column gap="2">
          <p style(mutedSm)>Anchored panel — Escape and light-dismiss from the platform.</p>
        </column>
      </popover>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Placement</label>
        <select size="sm" bind:value={placement} label="Placement">
          <option value="top">top</option>
          <option value="bottom">bottom</option>
          <option value="left">left</option>
          <option value="right">right</option>
          <option value="bottom-start">bottom-start</option>
          <option value="top-end">top-end</option>
        </select>
        <p style(mutedXs)>Free string on the host — common anchors listed for knobs.</p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <button size="sm" variant="outline" popovertarget="doc-pop-usage">Open popover</button>
      <popover id="doc-pop-usage" placement="bottom">
        <p style(mutedSm)>Panel content</p>
      </popover>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Basic anchor"
        lead="popovertarget + id — no refs."
        code={popExBasic()}
      >
        <button size="sm" variant="outline" popovertarget="doc-pop-ex1">Open</button>
        <popover id="doc-pop-ex1" placement="bottom">
          <p style(mutedSm)>Panel content</p>
        </popover>
      </doc-example>
      <doc-example
        title="Filters"
        lead="bottom-start hugs a toolbar control."
        code={popExFilter()}
      >
        <button size="sm" variant="outline" popovertarget="doc-pop-ex2">Filters</button>
        <popover id="doc-pop-ex2" placement="bottom-start">
          <p style(mutedSm)>Filter options</p>
        </popover>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
