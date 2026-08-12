<!--
  <doc-a> — first-class docs host for /components/a.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
  Dual mode: bare text link vs button-link chrome (component.button.*).
-->

<controller>
import type { Scope } from './doc-a.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const variant = signal('solid')
  const aTone = signal('primary')
  const aDisabled = signal(false)
  const pickATone = (t: string): void => aTone.set(t)
  const clearATone = (): void => aTone.set('')
  const aMeta = derived(() => {
    const t = aTone()
    const d = aDisabled() ? 'aria-disabled' : 'interactive'
    return `${variant()} · ${t || 'rest (button tokens)'} · ${d} · no component.a.*`
  })
  const entry = DETAILS.a
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const aExBare = () =>
    `<a href="/docs">text link</a>\n<!-- bare = freeform text link; freeform never stamps variant/tone -->`
  const aExVariants = () =>
    `<a href="/components" variant="default" tone="primary">default</a>\n<a href="/components" variant="solid" tone="primary">solid</a>\n<a href="/components" variant="soft" tone="primary">soft</a>\n<a href="/components" variant="outline" tone="primary">outline</a>\n<a href="/components" variant="ghost" tone="primary">ghost</a>`
  const aExRest = () =>
    `<a href="/docs" variant="default">default rest</a>\n<a href="/docs" variant="soft">soft rest</a>\n<!-- no tone → component.button.*; default ≠ solid invert -->`
  return {
    variant,
    aTone,
    aDisabled,
    pickATone,
    clearATone,
    aMeta,
    heading,
    blurb,
    props,
    usageCode,
    aExBare,
    aExVariants,
    aExRest,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
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
            <p style(monoXs)>{aMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
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
              <code>default</code> ≠ solid invert — rest uses <code>component.button.*</code>.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Tone</label>
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="rest — component.button tokens" placement="top">
                <div label="rest" on:click={clearATone} selected={!aTone() ? 'true' : null} style(swatch, swatchRest)></div>
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
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Bare text link</h3>
        <p style(exampleLead)>
          No <code>variant</code> attr — freeform never stamps variant/tone, so every
          href stays inline prose.
        </p>
        <row gap="3" align="center">
          <a href="/docs">Read the docs</a>
          <a href="/components">Browse components</a>
        </row>
        <code-block code={aExBare()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Button-link variants</h3>
        <p style(exampleLead)>
          Set <code>variant</code> for control chrome. Same family as button —
          <code>default</code> · <code>solid</code> · <code>soft</code> · <code>outline</code> ·
          <code>ghost</code>.
        </p>
        <row gap="2" align="center">
          <a href="/components" variant="default" tone="primary">default</a>
          <a href="/components" variant="solid" tone="primary">solid</a>
          <a href="/components" variant="soft" tone="primary">soft</a>
          <a href="/components" variant="outline" tone="primary">outline</a>
          <a href="/components" variant="ghost" tone="primary">ghost</a>
        </row>
        <code-block code={aExVariants()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Omit <code>tone</code> and button-link paint comes from
          <code>component.button.*</code> — no <code>component.a.*</code> invent.
        </p>
        <row gap="2" align="center">
          <a href="/docs" variant="default">default rest</a>
          <a href="/docs" variant="soft">soft rest</a>
          <a href="/docs" variant="outline">outline rest</a>
        </row>
        <code-block code={aExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
