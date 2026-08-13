<!--
  <doc-toast> — /components/toast
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style specimen = { box: size({ w: full, max-w: panel-max-w-sm }).space({ pad: 2 }).overflow({ all: visible }) }
  style stageCol = { box: size({ w: panel-max-w-md, max-w: full }) }
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style swatch = {
    box: size(7).flow({ shrink: 0 }),
    paint: radius(full).cursor(pointer).border({ color: transparent, width: 2 }).elevation(xs),
    when: { selected: { paint: border({ color: foreground }).elevation(sm) }, hover: { paint: filter({ opacity: 90 }) } },
}
  style swatchRest = { paint: fill({ color: background }).border({ color: border }) }
  style swatchPrimary = { paint: fill({ color: primary }) }
  style swatchInfo = { paint: fill({ color: info }) }
  style swatchSuccess = { paint: fill({ color: success }) }
  style swatchWarning = { paint: fill({ color: warning }) }
  style swatchDestructive = { paint: fill({ color: destructive }) }
  style swatchAccent = { paint: fill({ color: accent }) }
  style swatchMuted = { paint: fill({ color: muted }).border({ color: border }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
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
