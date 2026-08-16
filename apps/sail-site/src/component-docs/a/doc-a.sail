<!--
  <doc-a> — /components/a
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Dual mode: bare text link vs button-link chrome.
-->

<scope>
  style page = { size({ w: full, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style mutedXs = { fg(muted-foreground).type({ size: xs }) }
  style specimen = { pad(2).overflow({ all: visible }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style swatchRow = { size({ w: full }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchPrimary = { bg(primary) }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDestructive = { bg(destructive) }
  style swatchAccent = { bg(accent) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style swatchInfo = { bg(info) }
  style exampleBlock = { size({ w: full }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
  style demoRow = { size({ w: full }).pad({ y: 2 }) }
</scope>

<controller>
import type { Scope } from './doc-a.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const variant = signal('text')
  const aTone = signal('primary')
  const aDisabled = signal(false)
  const pickATone = (t: string): void => aTone.set(t)

  const entry = DETAILS.a
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = ['href="/components"']
    if (variant()) a.push(`variant="${variant()}"`)
    if (aTone()) a.push(`tone="${aTone()}"`)
    if (aDisabled()) a.push('aria-disabled="true"')
    return `<a ${a.join(' ')}>Open catalog</a>`
  })

  const aExBare = () =>
    `<a href="/docs">text link</a>\n<!-- bare / variant=text = no chrome; wrap a card or span -->`
  const aExWrap = () =>
    `<a href="/components/card">\n  <card>\n    <strong>Wrapped card</strong>\n    <p>The whole surface is the link.</p>\n  </card>\n</a>`
  const aExVariants = () =>
    `<a href="/components" variant="default" tone="primary">default</a>\n<a href="/components" variant="solid" tone="primary">solid</a>\n<a href="/components" variant="soft" tone="primary">soft</a>\n<a href="/components" variant="outline" tone="primary">outline</a>\n<a href="/components" variant="ghost" tone="primary">ghost</a>`
  const aExRest = () =>
    `<a href="/docs" variant="default">default</a>\n<a href="/docs" variant="soft">soft</a>\n<!-- no tone → button tokens; default ≠ solid invert -->`

  return {
    variant,
    aTone,
    aDisabled,
    pickATone,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    aExBare,
    aExWrap,
    aExVariants,
    aExRest,
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
      <if {aDisabled()}>
        <if {aTone()}>
          <a href="/components" variant={variant()} tone={aTone()} aria-disabled="true">Open catalog</a>
        {else =>
          <a href="/components" variant={variant()} aria-disabled="true">Open catalog</a>}</if>
      {else =>
        <if {aTone()}>
          <a href="/components" variant={variant()} tone={aTone()}>Open catalog</a>
        {else =>
          <a href="/components" variant={variant()}>Open catalog</a>}</if>}</if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Variant</label>
        <select size="sm" bind:value={variant} label="Variant">
          <option value="text">text</option>
          <option value="default">default</option>
          <option value="solid">solid</option>
          <option value="soft">soft</option>
          <option value="outline">outline</option>
          <option value="ghost">ghost</option>
        </select>
        <p style(mutedXs)>
          <code>text</code> (or omit) is a wrapping text link. Button chrome is
          opt-in. <code>default</code> ≠ solid invert.
        </p>
      </column>
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center" style(swatchRow)>
          <tooltip content="primary" placement="top">
            <div title="primary" label="primary" on:click={() => pickATone('primary')} selected={aTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
          </tooltip>
          <tooltip content="success" placement="top">
            <div title="success" label="success" on:click={() => pickATone('success')} selected={aTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
          </tooltip>
          <tooltip content="warning" placement="top">
            <div title="warning" label="warning" on:click={() => pickATone('warning')} selected={aTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
          </tooltip>
          <tooltip content="destructive" placement="top">
            <div title="destructive" label="destructive" on:click={() => pickATone('destructive')} selected={aTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
          </tooltip>
          <tooltip content="accent" placement="top">
            <div title="accent" label="accent" on:click={() => pickATone('accent')} selected={aTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
          </tooltip>
          <tooltip content="muted" placement="top">
            <div title="muted" label="muted" on:click={() => pickATone('muted')} selected={aTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
          </tooltip>
          <tooltip content="info" placement="top">
            <div title="info" label="info" on:click={() => pickATone('info')} selected={aTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
          </tooltip>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Disabled</label>
        <row gap="2" align="center">
          <switch bind:checked={aDisabled} size="sm" tone="primary"></switch>
          <span style(sizeValue)>{aDisabled() ? 'aria-disabled' : 'interactive'}</span>
        </row>
        <p style(mutedXs)>
          Anchors have no native disabled — host Style dim uses <code>aria-disabled</code>
          (or the <code>disabled</code> attr when stamped).
        </p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="center" style(specimen)>
      <a href="/components">Open catalog</a>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Bare text link"
        lead="No variant attr — freeform never stamps variant/tone, so every href stays inline prose. variant=text is the same skin."
        code={aExBare()}
      >
        <row gap="3" align="center" style(demoRow)>
          <a href="/docs">Read the docs</a>
          <a href="/components" variant="text">variant=text</a>
        </row>
      </doc-example>
      <doc-example
        title="Wrap another element"
        lead="Anchor is a wrapper. A card, span, or button-looking surface can be the link — don't invent a second link API."
        code={aExWrap()}
      >
        <a href="/components/card">
          <card style({ pad(3) })>
            <strong>Wrapped card</strong>
            <p style(mutedXs)>The whole surface is the link.</p>
          </card>
        </a>
      </doc-example>
      <doc-example
        title="Button-link variants"
        lead="Set variant for control chrome. Same family as button — default · solid · soft · outline · ghost."
        code={aExVariants()}
      >
        <row gap="2" align="center" style(demoRow)>
          <a href="/components" variant="default" tone="primary">default</a>
          <a href="/components" variant="solid" tone="primary">solid</a>
          <a href="/components" variant="soft" tone="primary">soft</a>
          <a href="/components" variant="outline" tone="primary">outline</a>
          <a href="/components" variant="ghost" tone="primary">ghost</a>
        </row>
      </doc-example>
      <doc-example
        title="Without tone"
        lead="Omit tone and button-link paint comes from shared button tokens."
        code={aExRest()}
      >
        <row gap="2" align="center" style(demoRow)>
          <a href="/docs" variant="default">default</a>
          <a href="/docs" variant="soft">soft</a>
          <a href="/docs" variant="outline">outline</a>
        </row>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
