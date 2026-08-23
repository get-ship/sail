<!--
  <doc-toast> — /components/toast
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { size({ w: 100%, max-w: breakpoint-desktop }).margin({ x: auto }).overflow({ all: visible }) }
  style specimen = { size({ w: 100%, max-w: panel-max-w-sm }).pad(2).overflow({ all: visible }) }
  style stageCol = { size({ w: panel-max-w-md, max-w: 100% }) }
  style mutedSm = { fg(muted-foreground).type({ size: sm }) }
  style knobLabel = { fg(muted-foreground).type({ size: xs, weight: medium, tracking: wide, case: uppercase }) }
  style swatch = { size(7).shrink(0).radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs), when: { selected: border({ color: foreground }).elevation(sm), hover: filter({ opacity: 90 }) } }
  style swatchPrimary = { bg(primary) }
  style swatchInfo = { bg(info) }
  style swatchSuccess = { bg(success) }
  style swatchWarning = { bg(warning) }
  style swatchDanger = { bg(danger) }
  style swatchAccent = { bg(accent) }
  style swatchMuted = { bg(muted).border({ color: border }) }
  style exampleBlock = { size({ w: 100% }).pad({ t: 2, b: 4 }).overflow({ all: visible }) }
  style sectionLabel = { fg(muted-foreground).type({ size: xs, tracking: wide, case: uppercase }) }
</scope>

<controller>
import type { Scope } from './doc-toast.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const toastTone = signal('info')
  const duration = signal('0')
  let toastSeq = 0
  const toastItems = signal<{ key: string }[]>([{ key: 'toast-0' }])
  const toastEmpty = derived(() => toastItems().length === 0)
  const exSuccess = signal(true)
  const exTones = signal(true)
  const exAction = signal(true)

  const pushToast = (): void => {
    toastSeq += 1
    toastItems.set([{ key: `toast-${toastSeq}` }])
  }
  const onDismiss = (): void => {
    toastItems.set([])
  }
  const pickTone = (t: string): void => {
    toastTone.set(t)
    pushToast()
  }
  const restoreSuccess = (): void => exSuccess.set(true)
  const onSuccessDismiss = (): void => exSuccess.set(false)
  const restoreTones = (): void => exTones.set(true)
  const onTonesDismiss = (): void => exTones.set(false)
  const restoreAction = (): void => exAction.set(true)
  const onActionDismiss = (): void => exAction.set(false)

  const entry = DETAILS.toast
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = [`tone="${toastTone()}"`]
    if (duration()) a.push(`duration="${duration()}"`)
    a.push('on-dismiss={onDismiss}')
    return `<toast ${a.join(' ')}>\n  <strong>Notice.</strong> Sticky when duration is 0.\n</toast>`
  })

  const toastExSuccess = () =>
    `<toast tone="success" duration="0" on-dismiss={onDismiss}>\n  <strong>Saved.</strong> Your changes are live.\n</toast>`
  const toastExTones = () =>
    `<toast tone="info">Info</toast>\n<toast tone="warning">Warning</toast>\n<toast tone="danger">Error</toast>`
  const toastExSticky = () =>
    `<!-- duration 0 = sticky; re-push from controller after dismiss -->\n<toast duration="0" on-dismiss={onDismiss}>Sticky</toast>`
  const toastExAction = () =>
    `<toast tone="info" duration="0" on-dismiss={onDismiss}>\n  <row align="center" gap="3">\n    <span>Invite sent to ada@ship.dev</span>\n    <button type="button" size="sm" variant="outline">Undo</button>\n  </row>\n</toast>`

  return {
    toastTone,
    duration,
    toastItems,
    toastEmpty,
    pushToast,
    onDismiss,
    pickTone,
    exSuccess,
    exTones,
    exAction,
    restoreSuccess,
    onSuccessDismiss,
    restoreTones,
    onTonesDismiss,
    restoreAction,
    onActionDismiss,
    toastExAction,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    toastExSuccess,
    toastExTones,
    toastExSticky,
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
      <if {toastEmpty()}>
        <button type="button" variant="outline" size="sm" on:click={pushToast} toast-show>Show toast</button>
      </if>
      <for {toastItems() as t} key={t.key}>
        <toast tone={toastTone()} duration={duration()} on-dismiss={onDismiss} style(stageCol)>
          <strong>Notice.</strong> Sticky when duration is 0.
        </toast>
      </for>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="primary"
            selected={toastTone() === 'primary' ? 'true' : null}
            on:click={() => pickTone('primary')}
            style(swatch, swatchPrimary)
          ></div>
          <div
            title="info"
            selected={toastTone() === 'info' ? 'true' : null}
            on:click={() => pickTone('info')}
            style(swatch, swatchInfo)
          ></div>
          <div
            title="success"
            selected={toastTone() === 'success' ? 'true' : null}
            on:click={() => pickTone('success')}
            style(swatch, swatchSuccess)
          ></div>
          <div
            title="warning"
            selected={toastTone() === 'warning' ? 'true' : null}
            on:click={() => pickTone('warning')}
            style(swatch, swatchWarning)
          ></div>
          <div
            title="danger"
            selected={toastTone() === 'danger' ? 'true' : null}
            on:click={() => pickTone('danger')}
            style(swatch, swatchDanger)
          ></div>
          <div
            title="accent"
            selected={toastTone() === 'accent' ? 'true' : null}
            on:click={() => pickTone('accent')}
            style(swatch, swatchAccent)
          ></div>
          <div
            title="muted"
            selected={toastTone() === 'muted' ? 'true' : null}
            on:click={() => pickTone('muted')}
            style(swatch, swatchMuted)
          ></div>
        </row>
      </column>
      <column gap="2">
        <label style(knobLabel)>Duration</label>
        <select size="sm" bind:value={duration} label="Duration">
          <option value="0">0 (sticky)</option>
          <option value="2000">2000</option>
          <option value="4000">4000</option>
          <option value="8000">8000</option>
        </select>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <toast tone="success" duration="0">
        <strong>Saved.</strong> Your changes are live.
      </toast>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Success sticky"
        lead='duration="0" + onDismiss — re-push from the controller after dismiss.'
        code={toastExSuccess()}
      >
        <if {exSuccess()}>
          <toast tone="success" duration="0" on-dismiss={onSuccessDismiss}>
            <strong>Saved.</strong> Your changes are live.
          </toast>
        {else =>
          <button type="button" variant="outline" size="sm" on:click={restoreSuccess}>Show toast</button>}</if>
      </doc-example>
      <doc-example title="Tone rails" lead="Tone paints the start stripe and a wash on the fill." code={toastExTones()}>
        <if {exTones()}>
          <column gap="2" style(specimen)>
            <toast tone="info" duration="0" on-dismiss={onTonesDismiss}>Info notice</toast>
            <toast tone="warning" duration="0" on-dismiss={onTonesDismiss}>Warning notice</toast>
          </column>
        {else =>
          <button type="button" variant="outline" size="sm" on:click={restoreTones}>Show toast</button>}</if>
      </doc-example>
      <doc-example
        title="With action"
        lead="Default slot can hold a control — no extra variant."
        code={toastExAction()}
      >
        <if {exAction()}>
          <toast tone="info" duration="0" on-dismiss={onActionDismiss} style(stageCol)>
            <row align="center" gap="3">
              <span>Invite sent to ada@ship.dev</span>
              <button type="button" size="sm" variant="outline">Undo</button>
            </row>
          </toast>
        {else =>
          <button type="button" variant="outline" size="sm" on:click={restoreAction}>Show toast</button>}</if>
      </doc-example>
      <doc-example
        title="Sticky note"
        lead="Dismiss removes the instance — push again to show."
        code={toastExSticky()}
      >
        <p style(mutedSm)>Use Show toast on the specimen to re-push after dismiss.</p>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
