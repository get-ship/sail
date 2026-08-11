<!--
  <doc-toast> — first-class docs host for /components/toast.
  Specimen left, knobs right — tone · duration · onDismiss.
-->

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
  const toastMeta = derived(() => {
    const t = toastTone() || 'rest'
    return `${t} · duration ${duration() || 'default'} · component.toast.*`
  })
  const entry = DETAILS.toast
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const toastExSuccess = () =>
    `<toast tone="success" duration="0" on-dismiss={onDismiss}>\n  <strong>Saved.</strong> Your changes are live.\n</toast>`
  const toastExTones = () =>
    `<toast tone="info">Info</toast>\n<toast tone="warning">Warning</toast>\n<toast tone="destructive">Error</toast>`
  const toastExSticky = () =>
    `<!-- duration 0 = sticky; re-push from controller after dismiss -->\n<toast duration="0" on-dismiss={onDismiss}>Sticky</toast>`
  return {
    toastTone, duration, toastItems, pushToast, onDismiss, pickTone, clearTone, toastMeta,
    heading, blurb, props, usageCode, toastExSuccess, toastExTones, toastExSticky,
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
  style stageCol = { box: w(panel-max-w-md).maxW(full) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style faceRow = { box: w(full) }
  style fullW = { box: w(full) }
  style swatchRow = { box: w(full) }
  style swatch = {
    box: size(6).shrink(0),
    paint: radius(full).borderWidth(1).border(border).cursor(pointer),
  }
  style swatchRest = { paint: bg(surface-1) }
  style swatchPrimary = { paint: bg(primary) }
  style swatchInfo = { paint: bg(info) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }

  style toastStage = { box: w(full).maxW(panel-max-w-sm) }
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
          <column gap="3" align="center" style(toastStage)>
            <button type="button" variant="outline" size="sm" on:click={pushToast}>Show toast</button>
            <for {toastItems() as t} key={t.key}>
              <if {toastTone()}>
                <toast tone={toastTone()} duration={duration()} on-dismiss={onDismiss} style(stageCol)>
                  <strong>Saved.</strong> Sticky when duration is 0.
                </toast>
              {else =>
                <toast duration={duration()} on-dismiss={onDismiss} style(stageCol)>
                  <strong>Notice.</strong> Rest paint from component.toast.*.
                </toast>}</if>
            </for>
            <p style(monoXs)>{toastMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Tone</label>
            <row gap="2" align="center" style(swatchRow)>
              <tooltip content="rest" placement="top">
                <div label="rest" on:click={clearTone} selected={!toastTone() ? 'true' : null} style(swatch, swatchRest)></div>
              </tooltip>
              <tooltip content="primary" placement="top">
                <div label="primary" on:click={() => pickTone('primary')} selected={toastTone() === 'primary' ? 'true' : null} style(swatch, swatchPrimary)></div>
              </tooltip>
              <tooltip content="info" placement="top">
                <div label="info" on:click={() => pickTone('info')} selected={toastTone() === 'info' ? 'true' : null} style(swatch, swatchInfo)></div>
              </tooltip>
              <tooltip content="success" placement="top">
                <div label="success" on:click={() => pickTone('success')} selected={toastTone() === 'success' ? 'true' : null} style(swatch, swatchSuccess)></div>
              </tooltip>
              <tooltip content="warning" placement="top">
                <div label="warning" on:click={() => pickTone('warning')} selected={toastTone() === 'warning' ? 'true' : null} style(swatch, swatchWarning)></div>
              </tooltip>
              <tooltip content="destructive" placement="top">
                <div label="destructive" on:click={() => pickTone('destructive')} selected={toastTone() === 'destructive' ? 'true' : null} style(swatch, swatchDestructive)></div>
              </tooltip>
              <tooltip content="accent" placement="top">
                <div label="accent" on:click={() => pickTone('accent')} selected={toastTone() === 'accent' ? 'true' : null} style(swatch, swatchAccent)></div>
              </tooltip>
              <tooltip content="muted" placement="top">
                <div label="muted" on:click={() => pickTone('muted')} selected={toastTone() === 'muted' ? 'true' : null} style(swatch, swatchMuted)></div>
              </tooltip>
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
            <p style(mutedXs)>ms string — 0 sticky. Hover pauses auto-dismiss.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.toast.bg / fg / border / stripe</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Success sticky</h3>
        <p style(exampleLead)><code>duration="0"</code> + <code>onDismiss</code> — re-push from controller.</p>
        <toast tone="success" duration="0">
          <strong>Saved.</strong> Your changes are live.
        </toast>
        <code-block code={toastExSuccess()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Tone rails</h3>
        <p style(exampleLead)>Semantic start-stripe per tone.</p>
        <column gap="2" style(toastStage)>
          <toast tone="info" duration="0">Info notice</toast>
          <toast tone="warning" duration="0">Warning notice</toast>
        </column>
        <code-block code={toastExTones()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Sticky note</h3>
        <p style(exampleLead)>Dismiss removes the instance — push again to show.</p>
        <code-block code={toastExSticky()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
