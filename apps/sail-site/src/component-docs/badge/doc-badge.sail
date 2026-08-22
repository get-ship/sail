<!--
  <doc-badge> — /components/badge
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style tickRow = { size({ w: 100% }).pad({ x: 1_5, t: 1 }) }
  style tick = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchPrimary = { bg(primary) }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDanger = { bg(danger) }
  style swatchAccent = { bg(accent) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style swatchInfo = { bg(info) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style demoRow = { size({ w: 100% }).pad({ y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-badge.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const BDG_SIZES = ['sm', 'md', 'lg'] as const
  const variant = signal('soft')
  const sizeIdx = signal(1)
  const badgeTone = signal('muted')
  const badgePill = signal(false)
  const pickBadgeTone = (t: string): void => badgeTone.set(t)

  const badgeSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), BDG_SIZES.length - 1)
    return BDG_SIZES[i] ?? 'md'
  })

  const entry = DETAILS.badge
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`variant="${variant()}"`]
    if (badgeSize() !== 'md') a.push(`size="${badgeSize()}"`)
    a.push(`tone="${badgeTone()}"`)
    if (badgePill()) a.push('pill')
    return `<badge ${a.join(' ')}>${badgeTone()}</badge>`
  })

  const bdgExStatus = () =>
    `<badge variant="soft" tone="success" size="sm">active</badge>\n<badge variant="outline" tone="muted" size="sm">away</badge>`
  const bdgExPill = () =>
    `<badge variant="solid" tone="primary" pill>12</badge>\n<badge variant="soft" tone="danger" pill>3</badge>`
  const bdgExDefault = () =>
    `<badge variant="soft" tone="muted">muted</badge>\n<badge variant="solid" tone="muted">muted</badge>\n<badge variant="outline" tone="muted">muted</badge>`

  return {
    variant,
    sizeIdx,
    badgeTone,
    badgePill,
    badgeSize,
    specimenCode,
    pickBadgeTone,
    heading,
    blurb,
    props,
    usageCode,
    bdgExStatus,
    bdgExPill,
    bdgExDefault,
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
    <column slot="specimen" gap="4" align="center">
      <badge
        variant={variant()}
        size={badgeSize()}
        tone={badgeTone()}
        pill={badgePill()}
      >
        {badgeTone()}
      </badge>
    </column>

    <column slot="knobs" gap="5" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Variant</label>
        <select size="sm" bind:value={variant} label="Variant">
          <option value="soft">soft</option>
          <option value="solid">solid</option>
          <option value="outline">outline</option>
        </select>
      </column>
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="primary"
            selected={badgeTone() === 'primary' ? 'true' : null}
            on:click={() => pickBadgeTone('primary')}
            style(swatch, swatchPrimary)
          ></div>
          <div
            title="success"
            selected={badgeTone() === 'success' ? 'true' : null}
            on:click={() => pickBadgeTone('success')}
            style(swatch, swatchSuccess)
          ></div>
          <div
            title="warning"
            selected={badgeTone() === 'warning' ? 'true' : null}
            on:click={() => pickBadgeTone('warning')}
            style(swatch, swatchWarning)
          ></div>
          <div
            title="danger"
            selected={badgeTone() === 'danger' ? 'true' : null}
            on:click={() => pickBadgeTone('danger')}
            style(swatch, swatchDanger)
          ></div>
          <div
            title="accent"
            selected={badgeTone() === 'accent' ? 'true' : null}
            on:click={() => pickBadgeTone('accent')}
            style(swatch, swatchAccent)
          ></div>
          <div
            title="muted"
            selected={badgeTone() === 'muted' ? 'true' : null}
            on:click={() => pickBadgeTone('muted')}
            style(swatch, swatchMuted)
          ></div>
          <div
            title="info"
            selected={badgeTone() === 'info' ? 'true' : null}
            on:click={() => pickBadgeTone('info')}
            style(swatch, swatchInfo)
          ></div>
        </row>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{badgeSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size" ticks="sm,md,lg"></slider>
      </column>
      <column gap="2">
        <label style(knobLabel)>Pill</label>
        <switch bind:checked={badgePill} size="sm" label="Pill"></switch>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center">
      <badge variant="soft" tone="primary">primary</badge>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <doc-example
        title="Status in a table cell"
        lead="Soft + tone for readable density. Default tone is muted."
        code={bdgExStatus()}
      >
        <row gap="2" align="center" style(demoRow)>
          <badge variant="soft" tone="success" size="sm">active</badge>
          <badge variant="outline" tone="muted" size="sm">away</badge>
          <badge variant="soft" tone="warning" size="sm">pending</badge>
        </row>
      </doc-example>

      <doc-example
        title="Pill counters"
        lead="pill sets full radius — counts, filters, and “new” marks."
        code={bdgExPill()}
      >
        <row gap="2" align="center" style(demoRow)>
          <badge variant="solid" tone="primary" pill>12</badge>
          <badge variant="soft" tone="danger" pill>3</badge>
          <badge variant="outline" tone="accent" pill>beta</badge>
        </row>
      </doc-example>

      <doc-example
        title="Default tone"
        lead="tone defaults to muted — a real colour, not the page background."
        code={bdgExDefault()}
      >
        <row gap="2" align="center" style(demoRow)>
          <badge variant="soft" tone="muted">muted</badge>
          <badge variant="solid" tone="muted">muted</badge>
          <badge variant="outline" tone="muted">muted</badge>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
