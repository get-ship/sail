<!--
  <doc-hover-card> — /components/hover-card
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Specimen: trigger + panel with placement knob.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style mutedXs = { fg(muted-foreground).type({ size: xs }) }
  style specimen = { pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-hover-card.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const placement = signal('top')

  const entry = DETAILS['hover-card']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const place = placement() !== 'top' ? ` placement="${placement()}"` : ' placement="top"'
    return `<a href="#" popovertarget="profile">@ada</a>\n<hover-card id="profile"${place}>\n  <strong>Ada Lovelace</strong>\n  <p>Mathematician · first programmer</p>\n</hover-card>`
  })

  const hcExBasic = () =>
    `<a href="#" popovertarget="profile">@ada</a>\n<hover-card id="profile" placement="top">\n  <strong>Ada Lovelace</strong>\n  <p>Mathematician · first programmer</p>\n</hover-card>`
  const hcExBottom = () =>
    `<hover-card id="h" placement="bottom">…</hover-card>`

  return {
    placement,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    hcExBasic,
    hcExBottom,
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
      <a href="#" popovertarget="doc-hc-live">@ada</a>
      <hover-card id="doc-hc-live" placement={placement()}>
        <strong>Ada Lovelace</strong>
        <p style(mutedXs)>Mathematician · first programmer</p>
      </hover-card>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Placement</label>
        <select size="sm" bind:value={placement} label="Placement">
          <option value="top">top</option>
          <option value="bottom">bottom</option>
          <option value="left">left</option>
          <option value="right">right</option>
          <option value="bottom-end">bottom-end</option>
        </select>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <a href="#" popovertarget="doc-hc-usage">@ada</a>
      <hover-card id="doc-hc-usage" placement="top">
        <strong>Ada Lovelace</strong>
        <p style(mutedXs)>Mathematician · first programmer</p>
      </hover-card>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Profile preview"
        lead="popovertarget names the hover-card id."
        code={hcExBasic()}
      >
        <a href="#" popovertarget="doc-hc-ex1">@ada</a>
        <hover-card id="doc-hc-ex1" placement="top">
          <strong>Ada Lovelace</strong>
          <p style(mutedXs)>Mathematician · first programmer</p>
        </hover-card>
      </doc-example>
      <doc-example
        title="Bottom placement"
        lead="Open below the trigger."
        code={hcExBottom()}
      >
        <a href="#" popovertarget="doc-hc-ex2">@grace</a>
        <hover-card id="doc-hc-ex2" placement="bottom">
          <strong>Grace Hopper</strong>
          <p style(mutedXs)>Rear Admiral · compiler pioneer</p>
        </hover-card>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
