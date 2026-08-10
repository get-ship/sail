<!--
  <doc-badge> — first-class docs host for /components/badge.
  Configurator stage + status / pill / rest examples.
-->

<controller>
import type { Scope } from './doc-badge.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const BDG_SIZES = ['sm', 'md', 'lg'] as const
  const variant = signal('soft')
  const sizeIdx = signal(1)
  const badgeTone = signal('primary')
  const badgePill = signal(false)
  const pickBadgeTone = (t: string): void => badgeTone.set(t)
  const clearBadgeTone = (): void => badgeTone.set('')

  const badgeSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), BDG_SIZES.length - 1)
    return BDG_SIZES[i] ?? 'md'
  })
  const badgeMeta = derived(() => {
    const t = badgeTone()
    return `${variant()} · ${badgeSize()} · ${t || 'rest (component tokens)'} · pill ${badgePill() ? 'on' : 'off'}`
  })

  const entry = DETAILS.badge
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const bdgExStatus = () =>
    `<badge variant="soft" tone="success" size="sm">active</badge>\n<badge variant="outline" tone="muted" size="sm">away</badge>`
  const bdgExPill = () =>
    `<badge variant="solid" tone="primary" pill>12</badge>\n<badge variant="soft" tone="destructive" pill>3</badge>`
  const bdgExRest = () =>
    `<badge variant="soft">rest tokens</badge>\n<!-- no tone → component.badge.* (themeable) -->`

  return {
    variant,
    sizeIdx,
    badgeTone,
    badgePill,
    badgeSize,
    badgeMeta,
    pickBadgeTone,
    clearBadgeTone,
    heading,
    blurb,
    props,
    usageCode,
    bdgExStatus,
    bdgExPill,
    bdgExRest,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style stageSplit = { box: w(full).minW(0).minH(64) }
  style stageCanvas = { box: grow(1).basis(0).minW(0).minH(64).pad(8) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style swatchRow = { box: w(full) }
  style tickRow = { box: w(full).padT(1) }
  style tick = {
    box: grow(1).basis(0).minW(0),
    paint: fg(muted-foreground),
    text: fontSize(xs).fontFamily(mono).textAlign(center),
  }
  style swatch = {
    box: w(7).h(7).shrink(0),
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
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
</scope>

<column gap="4" style(page)>
  <sail-component-viewer
    name={heading()}
    blurb={blurb()}
    pack="Component primitives"
    variant-attr=""
    usage={usageCode()}
    variants={[]}
    rows={props()}
  >
    <column slot="stage" gap="4" align="stretch" style({ box: w(full).minW(0) })>
      <row nowrap gap="0" align="stretch" style(stageSplit)>
        <center style(stageCanvas)>
          <column gap="4" align="center">
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
                rest
              </badge>}</if>
            <p style(monoXs)>{badgeMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
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
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="rest — component.badge tokens" placement="top">
                <div label="rest" on:click={clearBadgeTone} selected={!badgeTone() ? 'true' : null} style(swatch, swatchRest)></div>
              </tooltip>
              <tooltip content="primary" placement="top">
                <div label="primary" on:click={() => pickBadgeTone('primary')} selected={badgeTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
              </tooltip>
              <tooltip content="success" placement="top">
                <div label="success" on:click={() => pickBadgeTone('success')} selected={badgeTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
              </tooltip>
              <tooltip content="warning" placement="top">
                <div label="warning" on:click={() => pickBadgeTone('warning')} selected={badgeTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickBadgeTone('destructive')} selected={badgeTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
              <tooltip content="accent" placement="top">
                <div label="accent" on:click={() => pickBadgeTone('accent')} selected={badgeTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
              </tooltip>
              <tooltip content="muted" placement="top">
                <div label="muted" on:click={() => pickBadgeTone('muted')} selected={badgeTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
              </tooltip>
              <tooltip content="info" placement="top">
                <div label="info" on:click={() => pickBadgeTone('info')} selected={badgeTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
              </tooltip>
            </row>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{badgeSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>sm</span>
              <span style(tick)>md</span>
              <span style(tick)>lg</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Pill</label>
            <row gap="2" align="center">
              <switch bind:checked={badgePill} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{badgePill() ? 'full radius' : 'sm radius'}</span>
            </row>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Status in a table cell</h3>
        <p style(exampleLead)>
          Soft + tone for readable density. Omit tone when you want the theme’s
          <code>component.badge</code> tokens (rest swatch on the stage).
        </p>
        <row gap="2" align="center">
          <badge variant="soft" tone="success" size="sm">active</badge>
          <badge variant="outline" tone="muted" size="sm">away</badge>
          <badge variant="soft" tone="warning" size="sm">pending</badge>
        </row>
        <code-block code={bdgExStatus()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Pill counters</h3>
        <p style(exampleLead)>
          <code>pill</code> sets full radius — counts, filters, “new” marks.
        </p>
        <row gap="2" align="center">
          <badge variant="solid" tone="primary" pill>12</badge>
          <badge variant="soft" tone="destructive" pill>3</badge>
          <badge variant="outline" tone="accent" pill>beta</badge>
        </row>
        <code-block code={bdgExPill()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          No <code>tone</code> → soft/solid/outline paint from <code>component.badge.*</code>.
          Override those leaves in Studio themes to rebrand every rest badge.
        </p>
        <row gap="2" align="center">
          <badge variant="soft">soft rest</badge>
          <badge variant="solid">solid rest</badge>
          <badge variant="outline">outline rest</badge>
        </row>
        <code-block code={bdgExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
