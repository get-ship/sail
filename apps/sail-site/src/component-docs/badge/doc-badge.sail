<!--
  <doc-badge> — /components/badge
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style tickRow = { box: w(full).padT(1).padX(1_5) }
  style tick = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style swatch = {
    box: size(7).shrink(0),
    paint: radius(full).cursor(pointer).borderWidth(2).border(transparent).shadow(xs),
    when: {
      selected: { paint: border(foreground).shadow(sm) },
      hover: { paint: opacity(90) },
    },
  }
  style swatchPrimary = { paint: bg(primary) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted).border(border) }
  style swatchInfo = { paint: bg(info) }
  style swatchRest = { paint: bg(background).border(border) }
  style exampleBlock = { box: w(full).padT(2).padB(4) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style demoRow = { box: w(full).padY(2) }
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
  const clearBadgeTone = (): void => badgeTone.set('')

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
    if (badgeTone()) a.push(`tone="${badgeTone()}"`)
    if (badgePill()) a.push('pill')
    const label = badgeTone() || 'default'
    return `<badge ${a.join(' ')}>${label}</badge>`
  })

  const bdgExStatus = () =>
    `<badge variant="soft" tone="success" size="sm">active</badge>\n<badge variant="outline" tone="muted" size="sm">away</badge>`
  const bdgExPill = () =>
    `<badge variant="solid" tone="primary" pill>12</badge>\n<badge variant="soft" tone="destructive" pill>3</badge>`
  const bdgExDefault = () =>
    `<badge variant="soft">muted</badge>\n<badge variant="solid">muted</badge>\n<badge variant="outline">muted</badge>`

  return {
    variant,
    sizeIdx,
    badgeTone,
    badgePill,
    badgeSize,
    specimenCode,
    pickBadgeTone,
    clearBadgeTone,
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
      <if {badgeTone()}>
        <badge
          variant={variant()}
          size={badgeSize()}
          tone={badgeTone()}
          pill={badgePill()}
        >
          {badgeTone()}
        </badge>
      {else =>
        <badge variant={variant()} size={badgeSize()} pill={badgePill()}>
          default
        </badge>}</if>
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
            title="default (theme)"
            selected={!badgeTone() ? 'true' : null}
            on:click={clearBadgeTone}
            style(swatch, swatchRest)
          ></div>
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
            title="destructive"
            selected={badgeTone() === 'destructive' ? 'true' : null}
            on:click={() => pickBadgeTone('destructive')}
            style(swatch, swatchDestructive)
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
        lead="Soft + tone for readable density. Omit tone for the theme default."
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
          <badge variant="soft" tone="destructive" pill>3</badge>
          <badge variant="outline" tone="accent" pill>beta</badge>
        </row>
      </doc-example>

      <doc-example
        title="Default (theme)"
        lead="Default tone is muted — a real colour, not the page background."
        code={bdgExDefault()}
      >
        <row gap="2" align="center" style(demoRow)>
          <badge variant="soft">muted</badge>
          <badge variant="solid">muted</badge>
          <badge variant="outline">muted</badge>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
