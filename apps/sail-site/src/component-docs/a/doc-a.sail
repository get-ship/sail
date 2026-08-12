<!--
  <doc-a> — /components/a
  Shared chrome: split + snippet copy + usage Code|Live|Split.
  Dual mode: bare text link vs button-link chrome.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style specimen = { box: pad(2).overflow(visible) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style swatchRow = { box: w(full) }
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
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
  style demoRow = { box: w(full).padY(2) }
</scope>

<controller>
import type { Scope } from './doc-a.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const variant = signal('solid')
  const aTone = signal('primary')
  const aDisabled = signal(false)
  const pickATone = (t: string): void => aTone.set(t)
  const clearATone = (): void => aTone.set('')

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
    `<a href="/docs">text link</a>\n<!-- bare = freeform text link; freeform never stamps variant/tone -->`
  const aExVariants = () =>
    `<a href="/components" variant="default" tone="primary">default</a>\n<a href="/components" variant="solid" tone="primary">solid</a>\n<a href="/components" variant="soft" tone="primary">soft</a>\n<a href="/components" variant="outline" tone="primary">outline</a>\n<a href="/components" variant="ghost" tone="primary">ghost</a>`
  const aExRest = () =>
    `<a href="/docs" variant="default">default</a>\n<a href="/docs" variant="soft">soft</a>\n<!-- no tone → button tokens; default ≠ solid invert -->`

  return {
    variant,
    aTone,
    aDisabled,
    pickATone,
    clearATone,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    aExBare,
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
          <option value="default">default</option>
          <option value="solid">solid</option>
          <option value="soft">soft</option>
          <option value="outline">outline</option>
          <option value="ghost">ghost</option>
        </select>
        <p style(mutedXs)>
          Omit <code>variant</code> for a bare text link. Set it for button-link chrome.
          <code>default</code> ≠ solid invert.
        </p>
      </column>
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center" style(swatchRow)>
          <tooltip content="no tone" placement="top">
            <div label="none" on:click={clearATone} selected={!aTone() ? 'true' : null} style(swatch, swatchRest)></div>
          </tooltip>
          <tooltip content="primary" placement="top">
            <div label="primary" on:click={() => pickATone('primary')} selected={aTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
          </tooltip>
          <tooltip content="success" placement="top">
            <div label="success" on:click={() => pickATone('success')} selected={aTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
          </tooltip>
          <tooltip content="warning" placement="top">
            <div label="warning" on:click={() => pickATone('warning')} selected={aTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
          </tooltip>
          <tooltip content="destructive" placement="top">
            <div label="destructive" on:click={() => pickATone('destructive')} selected={aTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
          </tooltip>
          <tooltip content="accent" placement="top">
            <div label="accent" on:click={() => pickATone('accent')} selected={aTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
          </tooltip>
          <tooltip content="muted" placement="top">
            <div label="muted" on:click={() => pickATone('muted')} selected={aTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
          </tooltip>
          <tooltip content="info" placement="top">
            <div label="info" on:click={() => pickATone('info')} selected={aTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
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
      <a href="/components" variant="solid" tone="primary">Open catalog</a>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Bare text link"
        lead="No variant attr — freeform never stamps variant/tone, so every href stays inline prose."
        code={aExBare()}
      >
        <row gap="3" align="center" style(demoRow)>
          <a href="/docs">Read the docs</a>
          <a href="/components">Browse components</a>
        </row>
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
