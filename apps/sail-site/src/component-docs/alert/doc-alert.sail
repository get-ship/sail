<!--
  <doc-alert> — first-class docs host for /components/alert.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-alert.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {

  const alertTone = signal('info')
  const alertDismissible = signal(false)
  const alertVisible = signal(true)
  const pickAlertTone = (t: string): void => { alertTone.set(t); alertVisible.set(true) }
  const clearAlertTone = (): void => { alertTone.set(''); alertVisible.set(true) }
  const onAlertDismiss = (): void => alertVisible.set(false)
  const restoreAlert = (): void => alertVisible.set(true)
  const alertMeta = derived(() => {
    const t = alertTone()
    const d = alertDismissible() ? 'dismissible' : 'sticky'
    const live = t === 'warning' || t === 'destructive' ? 'assertive alert' : 'polite status'
    return `${t || 'rest (component tokens)'} · ${d} · ${live}${alertVisible() ? '' : ' · dismissed'}`
  })
  const alertTitle = derived(() => {
    switch (alertTone()) {
      case 'success': return 'Saved'
      case 'warning': return 'Check your input'
      case 'destructive': return 'Something went wrong'
      case 'muted': return 'Note'
      case 'info': return 'Heads up'
      default: return 'Notice'
    }
  })
  const alertBody = derived(() => {
    switch (alertTone()) {
      case 'success': return 'All changes were written successfully.'
      case 'warning': return 'Two fields need attention before you continue.'
      case 'destructive': return 'We could not reach the server. Try again.'
      case 'muted': return 'Neutral callout painted from component.alert tokens and muted stripe.'
      case 'info': return 'A new version is available. Refresh when you get a chance.'
      default: return 'Rest paint uses component.alert.* leaves — themeable in Studio.'
    }
  })
  const entry = DETAILS.alert
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const alrtExTones = () =>
    `<alert tone="info">\n  <strong slot="title">Heads up</strong>\n  A new version is available.\n</alert>\n<alert tone="success">\n  <strong slot="title">Saved</strong>\n  All changes were written.\n</alert>`
  const alrtExDismiss = () =>
    `<alert tone="warning" dismissible onDismiss={onDismiss}>\n  <strong slot="title">Check your input</strong>\n  Two fields need attention.\n</alert>`
  const alrtExRest = () =>
    `<alert>\n  <strong slot="title">Notice</strong>\n  Rest tokens — component.alert.*\n</alert>`
  return {
    alertTone, alertDismissible, alertVisible, alertMeta, alertTitle, alertBody,
    pickAlertTone, clearAlertTone, onAlertDismiss, restoreAlert,
    heading, blurb, props, usageCode, alrtExTones, alrtExDismiss, alrtExRest,
  }

})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style fullW = { box: w(full) }
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
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
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
          <column gap="4" align="stretch" style(fullW)>
            <if {alertVisible()}>
              <if {alertTone()}>
                <alert tone={alertTone()} dismissible={alertDismissible()} on-dismiss={onAlertDismiss} style(fullW)>
                  <strong slot="title">{alertTitle()}</strong>
                  {alertBody()}
                </alert>
              {else =>
                <alert dismissible={alertDismissible()} on-dismiss={onAlertDismiss} style(fullW)>
                  <strong slot="title">{alertTitle()}</strong>
                  {alertBody()}
                </alert>}</if>
            {else =>
              <column gap="3" align="center">
                <p style(mutedSm)>Dismissed — restore to try the knobs again.</p>
                <button type="button" variant="outline" size="sm" on:click={restoreAlert}>Restore alert</button>
              </column>}</if>
            <p style(monoXs)>{alertMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Tone</label>
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="rest — component.alert tokens" placement="top">
                <div label="rest" on:click={clearAlertTone} selected={!alertTone() ? 'true' : null} style(swatch, swatchRest)></div>
              </tooltip>
              <tooltip content="info" placement="top">
                <div label="info" on:click={() => pickAlertTone('info')} selected={alertTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
              </tooltip>
              <tooltip content="success" placement="top">
                <div label="success" on:click={() => pickAlertTone('success')} selected={alertTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
              </tooltip>
              <tooltip content="warning" placement="top">
                <div label="warning" on:click={() => pickAlertTone('warning')} selected={alertTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickAlertTone('destructive')} selected={alertTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
              <tooltip content="muted" placement="top">
                <div label="muted" on:click={() => pickAlertTone('muted')} selected={alertTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
              </tooltip>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Dismissible</label>
            <row gap="2" align="center">
              <switch bind:checked={alertDismissible} size="sm" tone="primary"></switch>
              <span style(sizeValue)>{alertDismissible() ? 'close control on' : 'sticky'}</span>
            </row>
          </column>
        </column>
      </row>
    </column>
    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Status tones</h3>
        <p style(exampleLead)>Tone brands the left stripe and icon. warning/destructive flip liveRegion to assertive.</p>
        <column gap="2" style(fullW)>
          <alert tone="info"><strong slot="title">Heads up</strong>A new version is available.</alert>
          <alert tone="success"><strong slot="title">Saved</strong>All changes were written.</alert>
        </column>
        <code-block code={alrtExTones()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Dismissible</h3>
        <p style(exampleLead)><code>dismissible</code> shows the close control; <code>onDismiss</code> fires before remove.</p>
        <alert tone="warning" dismissible>
          <strong slot="title">Check your input</strong>
          Two fields need attention before you continue.
        </alert>
        <code-block code={alrtExDismiss()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>No <code>tone</code> → bg/fg/border/stripe from <code>component.alert.*</code>.</p>
        <alert><strong slot="title">Notice</strong>Rest paint — component.alert.* leaves.</alert>
        <code-block code={alrtExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
