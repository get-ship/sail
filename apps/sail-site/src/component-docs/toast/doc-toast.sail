<!--
  <doc-toast> — /components/toast
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(full).maxW(panel-max-w-sm).pad(2).overflow(visible) }
  style stageCol = { box: w(panel-max-w-md).maxW(full) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style swatch = {
    box: size(7).shrink(0),
    paint: radius(full).cursor(pointer).borderWidth(2).border(transparent).shadow(xs),
    when: {
      selected: { paint: border(foreground).shadow(sm) },
      hover: { paint: opacity(90) },
    },
  }
  style swatchRest = { paint: bg(background).border(border) }
  style swatchPrimary = { paint: bg(primary) }
  style swatchInfo = { paint: bg(info) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted).border(border) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
</scope>

<controller>
import type { Scope } from './doc-toast.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const toastTone = signal('success')
  const duration = signal('0')
  let toastSeq = 0
  const toastItems = signal<{ key: string }[]>([{ key: 'toast-0' }])

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
  const clearTone = (): void => {
    toastTone.set('')
    pushToast()
  }

  const entry = DETAILS.toast
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const a: string[] = []
    if (toastTone()) a.push(`tone="${toastTone()}"`)
    if (duration()) a.push(`duration="${duration()}"`)
    a.push('on-dismiss={onDismiss}')
    const body = toastTone()
      ? '<strong>Saved.</strong> Sticky when duration is 0.'
      : '<strong>Notice.</strong> Default theme colors when tone is omitted.'
    return `<toast ${a.join(' ')}>\n  ${body}\n</toast>`
  })

  const toastExSuccess = () =>
    `<toast tone="success" duration="0" on-dismiss={onDismiss}>\n  <strong>Saved.</strong> Your changes are live.\n</toast>`
  const toastExTones = () =>
    `<toast tone="info">Info</toast>\n<toast tone="warning">Warning</toast>\n<toast tone="destructive">Error</toast>`
  const toastExSticky = () =>
    `<!-- duration 0 = sticky; re-push from controller after dismiss -->\n<toast duration="0" on-dismiss={onDismiss}>Sticky</toast>`

  return {
    toastTone,
    duration,
    toastItems,
    pushToast,
    onDismiss,
    pickTone,
    clearTone,
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
      <button type="button" variant="outline" size="sm" on:click={pushToast}>Show toast</button>
      <for {toastItems() as t} key={t.key}>
        <if {toastTone()}>
          <toast tone={toastTone()} duration={duration()} on-dismiss={onDismiss} style(stageCol)>
            <strong>Saved.</strong> Sticky when duration is 0.
          </toast>
        {else =>
          <toast duration={duration()} on-dismiss={onDismiss} style(stageCol)>
            <strong>Notice.</strong> Default theme colors when tone is omitted.
          </toast>}</if>
      </for>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Tone</label>
        <row gap="2" align="center">
          <div
            title="default"
            selected={!toastTone() ? 'true' : null}
            on:click={clearTone}
            style(swatch, swatchRest)
          ></div>
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
            title="destructive"
            selected={toastTone() === 'destructive' ? 'true' : null}
            on:click={() => pickTone('destructive')}
            style(swatch, swatchDestructive)
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
        <toast tone="success" duration="0">
          <strong>Saved.</strong> Your changes are live.
        </toast>
      </doc-example>
      <doc-example title="Tone rails" lead="Semantic start-stripe per tone." code={toastExTones()}>
        <column gap="2" style(specimen)>
          <toast tone="info" duration="0">Info notice</toast>
          <toast tone="warning" duration="0">Warning notice</toast>
        </column>
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
