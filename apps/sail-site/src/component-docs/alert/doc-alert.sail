<!--
  <doc-alert> — /components/alert
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<controller>
import type { Scope } from './doc-alert.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const alertTone = signal('info')
  const alertDismissible = signal(false)
  const alertVisible = signal(true)

  const pickAlertTone = (t: string): void => {
    alertTone.set(t)
    alertVisible.set(true)
  }
  const clearAlertTone = (): void => {
    alertTone.set('')
    alertVisible.set(true)
  }
  const onAlertDismiss = (): void => alertVisible.set(false)
  const restoreAlert = (): void => alertVisible.set(true)

  const alertTitle = derived(() => {
    switch (alertTone()) {
      case 'success':
        return 'Saved'
      case 'warning':
        return 'Check your input'
      case 'destructive':
        return 'Something went wrong'
      case 'muted':
        return 'Note'
      case 'info':
        return 'Heads up'
      default:
        return 'Notice'
    }
  })
  const alertBody = derived(() => {
    switch (alertTone()) {
      case 'success':
        return 'All changes were written successfully.'
      case 'warning':
        return 'Two fields need attention before you continue.'
      case 'destructive':
        return 'We could not reach the server. Try again.'
      case 'muted':
        return 'A neutral callout with default theme colors.'
      case 'info':
        return 'A new version is available. Refresh when you get a chance.'
      default:
        return 'Inline status with default theme colors — no tone set.'
    }
  })

  const entry = DETAILS.alert
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (alertTone()) a.push(`tone="${alertTone()}"`)
    if (alertDismissible()) a.push('dismissible')
    if (alertDismissible()) a.push('onDismiss={onDismiss}')
    const open = a.length ? `<alert ${a.join(' ')}>` : `<alert>`
    return `${open}\n  <strong slot="title">${alertTitle()}</strong>\n  ${alertBody()}\n</alert>`
  })

  const alrtExTones = () =>
    `<alert tone="info">\n  <strong slot="title">Heads up</strong>\n  A new version is available.\n</alert>\n<alert tone="success">\n  <strong slot="title">Saved</strong>\n  All changes were written.\n</alert>`
  const alrtExDismiss = () =>
    `<alert tone="warning" dismissible onDismiss={onDismiss}>\n  <strong slot="title">Check your input</strong>\n  Two fields need attention.\n</alert>`
  const alrtExDefault = () =>
    `<alert>\n  <strong slot="title">Notice</strong>\n  Default theme colors when tone is omitted.\n</alert>`

  return {
    alertTone,
    alertDismissible,
    alertVisible,
    alertTitle,
    alertBody,
    pickAlertTone,
    clearAlertTone,
    onAlertDismiss,
    restoreAlert,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    alrtExTones,
    alrtExDismiss,
    alrtExDefault,
  }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(full).maxW(panel-max-w-md).pad(2).overflow(visible) }
  style fullW = { box: w(full) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style swatch = {
    box: size(7).shrink(0),
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
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
</scope>

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
      <if {alertVisible()}>
        <if {alertTone()}>
          <alert
            tone={alertTone()}
            dismissible={alertDismissible()}
            on-dismiss={onAlertDismiss}
            style(fullW)
          >
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
          <button type="button" variant="outline" size="sm" on:click={restoreAlert}>
            Restore alert
          </button>
        </column>}</if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="default"
            selected={!alertTone() ? 'true' : null}
            on:click={clearAlertTone}
            style(swatch, swatchRest)
          ></div>
          <div
            title="info"
            selected={alertTone() === 'info' ? 'true' : null}
            on:click={() => pickAlertTone('info')}
            style(swatch, swatchInfo)
          ></div>
          <div
            title="success"
            selected={alertTone() === 'success' ? 'true' : null}
            on:click={() => pickAlertTone('success')}
            style(swatch, swatchSuccess)
          ></div>
          <div
            title="warning"
            selected={alertTone() === 'warning' ? 'true' : null}
            on:click={() => pickAlertTone('warning')}
            style(swatch, swatchWarning)
          ></div>
          <div
            title="destructive"
            selected={alertTone() === 'destructive' ? 'true' : null}
            on:click={() => pickAlertTone('destructive')}
            style(swatch, swatchDestructive)
          ></div>
          <div
            title="muted"
            selected={alertTone() === 'muted' ? 'true' : null}
            on:click={() => pickAlertTone('muted')}
            style(swatch, swatchMuted)
          ></div>
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

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <alert tone="info" style(fullW)>
        <strong slot="title">Heads up</strong>
        A new version is available. Refresh when you get a chance.
      </alert>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Status tones"
        lead="Tone brands the left stripe and icon. warning and destructive use an assertive live region."
        code={alrtExTones()}
      >
        <column gap="2" style(fullW)>
          <alert tone="info">
            <strong slot="title">Heads up</strong>
            A new version is available.
          </alert>
          <alert tone="success">
            <strong slot="title">Saved</strong>
            All changes were written.
          </alert>
        </column>
      </doc-example>
      <doc-example
        title="Dismissible"
        lead="dismissible shows the close control; onDismiss fires before remove."
        code={alrtExDismiss()}
      >
        <alert tone="warning" dismissible style(fullW)>
          <strong slot="title">Check your input</strong>
          Two fields need attention before you continue.
        </alert>
      </doc-example>
      <doc-example
        title="Default theme colors"
        lead="Omit tone for default theme colors."
        code={alrtExDefault()}
      >
        <alert style(fullW)>
          <strong slot="title">Notice</strong>
          Default theme colors when tone is omitted.
        </alert>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
