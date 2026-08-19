<!--
  <doc-alert> — /components/alert
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 100%, max-w: overlay-max-w-md }).pad(2).overflow({ all: visible }) }
  style fullW = { size({ w: 100% }) }
  style mutedSm = { fg(muted-foreground).type({ size: sm }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style sizeValue = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDestructive = { bg(destructive) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style swatchInfo = { bg(info) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

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
        return 'Heads up'
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
        return 'A neutral callout with muted tone.'
      case 'info':
        return 'A new version is available. Refresh when you get a chance.'
      default:
        return 'A new version is available. Refresh when you get a chance.'
    }
  })

  const entry = DETAILS.alert
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`tone="${alertTone()}"`]
    if (alertDismissible()) a.push('dismissible')
    if (alertDismissible()) a.push('onDismiss={onDismiss}')
    return `<alert ${a.join(' ')}>\n  <strong slot="title">${alertTitle()}</strong>\n  ${alertBody()}\n</alert>`
  })

  const alrtExTones = () =>
    `<alert tone="info">\n  <strong slot="title">Heads up</strong>\n  A new version is available.\n</alert>\n<alert tone="success">\n  <strong slot="title">Saved</strong>\n  All changes were written.\n</alert>`
  const alrtExDismiss = () =>
    `<alert tone="warning" dismissible onDismiss={onDismiss}>\n  <strong slot="title">Check your input</strong>\n  Two fields need attention.\n</alert>`
  const alrtExDefault = () =>
    `<alert tone="info">\n  <strong slot="title">Heads up</strong>\n  tone defaults to info.\n</alert>`

  return {
    alertTone,
    alertDismissible,
    alertVisible,
    alertTitle,
    alertBody,
    pickAlertTone,
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
        title="Default tone"
        lead="tone defaults to info — a bare alert is a finished status banner."
        code={alrtExDefault()}
      >
        <alert tone="info" style(fullW)>
          <strong slot="title">Heads up</strong>
          tone defaults to info.
        </alert>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
