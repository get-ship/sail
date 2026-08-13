<!--
  <doc-accordion> — /components/accordion
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }) }
  style specimen = { box: size({ w: panel-max-w-md, max-w: full }) }
  style fullW = { box: size({ w: full }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style mutedXs = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style sizeValue = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }) }
  style sectionLabel = { paint: ink({ color: muted-foreground }), text: type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-accordion.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const open = signal(false)
  const disabled = signal(false)
  const name = signal('')
  const multiple = signal(false)
  const lastOpen = signal('')
  const onToggle = (detail: { open?: boolean }): void => {
    const o = detail?.open
    if (typeof o === 'boolean') {
      open.set(o)
      lastOpen.set(o ? 'open' : 'closed')
    }
  }

  const entry = DETAILS.accordion
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (open()) a.push('open')
    if (disabled()) a.push('disabled')
    if (name()) a.push(`name="${name()}"`)
    if (multiple()) a.push('multiple')
    const openTag = a.length ? `<accordion ${a.join(' ')}>` : `<accordion>`
    return `${openTag}\n  <span slot="trigger">Webhook endpoints</span>\n  <p>POST /v1/hooks · signed with your workspace secret.</p>\n</accordion>`
  })

  const accExBasic = () =>
    `<accordion>\n  <span slot="trigger">Webhook endpoints</span>\n  <p>POST /v1/hooks · signed with your workspace secret.</p>\n</accordion>`
  const accExOpen = () =>
    `<accordion open>\n  <span slot="trigger">Already open</span>\n  <p>Content visible on first paint.</p>\n</accordion>`
  const accExExclusive = () =>
    `<accordion name="faq">…</accordion>\n<accordion name="faq">…</accordion>\n<!-- shared name → exclusive (native details name) -->`
  const accExMultiple = () =>
    `<accordion multiple name="filters">…</accordion>\n<!-- multiple allows several open at once -->`

  return {
    open,
    disabled,
    name,
    multiple,
    lastOpen,
    onToggle,
    specimenCode,
    heading,
    blurb,
    props,
    usageCode,
    accExBasic,
    accExOpen,
    accExExclusive,
    accExMultiple,
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
      <accordion
        open={open() ? 'true' : null}
        disabled={disabled() ? 'true' : null}
        name={name() || null}
        multiple={multiple() ? 'true' : null}
        on-toggle={onToggle}
        style(fullW)
      >
        <span slot="trigger">Webhook endpoints</span>
        <p style(mutedSm)>POST /v1/hooks · signed with your workspace secret.</p>
      </accordion>
      <if {name()}>
        <accordion name={name()} multiple={multiple() ? 'true' : null} style(fullW)>
          <span slot="trigger">Sibling (same name group)</span>
          <p style(mutedSm)>Shared name = exclusive unless multiple.</p>
        </accordion>
      </if>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Open</label>
        <row gap="2" align="center">
          <switch bind:checked={open} size="sm" tone="primary" label="Open"></switch>
          <span style(sizeValue)>{open() ? 'expanded' : 'collapsed'}</span>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Disabled</label>
        <switch bind:checked={disabled} size="sm" tone="primary" label="Disabled"></switch>
      </column>
      <column gap="2">
        <label style(knobLabel)>Name</label>
        <select size="sm" bind:value={name} label="Name">
          <option value="">(none — independent)</option>
          <option value="faq">faq</option>
          <option value="filters">filters</option>
        </select>
        <p style(mutedXs)>Shared name → exclusive multi-item (native details name).</p>
      </column>
      <column gap="2">
        <label style(knobLabel)>Multiple</label>
        <row gap="2" align="center">
          <switch bind:checked={multiple} size="sm" tone="primary" label="Multiple"></switch>
          <span style(sizeValue)>{multiple() ? 'several open' : 'exclusive when named'}</span>
        </row>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <accordion style(fullW)>
        <span slot="trigger">Webhook endpoints</span>
        <p style(mutedSm)>POST /v1/hooks · signed with your workspace secret.</p>
      </accordion>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Basic disclosure"
        lead="Trigger slot always visible; body expands on toggle."
        code={accExBasic()}
      >
        <accordion style(fullW)>
          <span slot="trigger">Webhook endpoints</span>
          <p style(mutedSm)>POST /v1/hooks · signed with your workspace secret.</p>
        </accordion>
      </doc-example>

      <doc-example
        title="Open by default"
        lead="open reflects initial expanded state."
        code={accExOpen()}
      >
        <accordion open style(fullW)>
          <span slot="trigger">Already open</span>
          <p style(mutedSm)>Content visible on first paint.</p>
        </accordion>
      </doc-example>

      <doc-example
        title="Exclusive group"
        lead="Shared name — only one open at a time."
        code={accExExclusive()}
      >
        <column gap="2" style(fullW)>
          <accordion name="faq-demo" style(fullW)>
            <span slot="trigger">What is a webhook?</span>
            <p style(mutedSm)>An HTTP callback your app receives on events.</p>
          </accordion>
          <accordion name="faq-demo" style(fullW)>
            <span slot="trigger">How do I verify?</span>
            <p style(mutedSm)>Check the signature against your workspace secret.</p>
          </accordion>
        </column>
      </doc-example>

      <doc-example
        title="Multiple open"
        lead="multiple when several may stay open together."
        code={accExMultiple()}
      >
        <column gap="2" style(fullW)>
          <accordion multiple name="filters-demo" open style(fullW)>
            <span slot="trigger">Status</span>
            <p style(mutedSm)>Active, paused, archived.</p>
          </accordion>
          <accordion multiple name="filters-demo" open style(fullW)>
            <span slot="trigger">Region</span>
            <p style(mutedSm)>US, EU, APAC.</p>
          </accordion>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
