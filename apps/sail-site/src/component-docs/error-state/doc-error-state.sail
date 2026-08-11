<!--
  <doc-error-state> — first-class docs host for /components/error-state.
  Specimen left, knobs right — variant × size × code chip.
-->

<controller>
import type { Scope } from './doc-error-state.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const ER_SIZES = ['sm', 'md', 'lg'] as const
  const sizeIdx = signal(1)
  const variant = signal('default')
  const errCode = signal('ERR_TIMEOUT')
  const erSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), ER_SIZES.length - 1)
    return ER_SIZES[i] ?? 'md'
  })
  const erMeta = derived(() => {
    const c = errCode()
    return `${variant()} · ${erSize()} · code ${c || '(hidden)'} · component.error-state.*`
  })
  const entry = DETAILS['error-state']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const erExDefault = () =>
    `<error-state code="ERR_TIMEOUT">\n  <strong slot="title">Could not load your data</strong>\n  The request failed after 3 retries.\n  <button slot="action" variant="solid" tone="destructive" size="sm">Retry</button>\n</error-state>`
  const erExInline = () =>
    `<error-state variant="inline">\n  <strong slot="title">Partial failure</strong>\n  One pane could not refresh.\n  <button slot="action" variant="outline" size="sm">Retry</button>\n</error-state>`
  const erExRest = () =>
    `<error-state>\n  <strong slot="title">Something went wrong</strong>\n  Rest chrome → component.error-state.*\n</error-state>`
  return {
    sizeIdx,
    variant,
    errCode,
    erSize,
    erMeta,
    heading,
    blurb,
    props,
    usageCode,
    erExDefault,
    erExInline,
    erExRest,
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
  style fullW = { box: w(full) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style tickRow = { box: w(full).padT(1) }
  style tick = {
    box: grow(1).basis(0).minW(0),
    paint: fg(muted-foreground),
    text: fontSize(xs).fontFamily(mono).textAlign(center),
  }
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
          <column gap="3" align="stretch" style(stageCol)>
            <error-state code={errCode()} variant={variant()} size={erSize()} style(fullW)>
              <strong slot="title">Could not load your data</strong>
              The request failed after 3 retries. Check your connection and try again.
              <span slot="details">at fetchData (api.ts:42)
  at load (dashboard.ts:19)
  Error: gateway timeout</span>
              <button slot="action" type="button" variant="solid" tone="destructive" size="sm">Retry</button>
            </error-state>
            <p style(monoXs)>{erMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Variant</label>
            <select size="sm" bind:value={variant} label="Variant">
              <option value="default">default</option>
              <option value="inline">inline</option>
            </select>
            <p style(mutedXs)>
              default = centered card on component leaves; inline = start-aligned with
              destructive border + muted fill.
            </p>
          </column>
          <column gap="2">
            <row gap="2" align="center" justify="between">
              <label style(knobLabel)>Size</label>
              <span style(sizeValue)>{erSize()}</span>
            </row>
            <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>sm</span>
              <span style(tick)>md</span>
              <span style(tick)>lg</span>
            </row>
          </column>
          <column gap="2">
            <label style(knobLabel)>Code</label>
            <input type="text" bind:value={errCode} size="sm" placeholder="ERR_…" style(fullW) />
            <p style(mutedXs)>
              Machine-readable chip. Empty hides the code region via
              <code>use:recoverySurface</code>.
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Default recovery</h3>
        <p style(exampleLead)>
          role=alert · code chip · details gate · action focus via
          <code>use:recoverySurface</code>.
        </p>
        <error-state code="ERR_TIMEOUT" style(stageCol)>
          <strong slot="title">Could not load your data</strong>
          The request failed after 3 retries.
          <button slot="action" type="button" variant="solid" tone="destructive" size="sm">Retry</button>
        </error-state>
        <code-block code={erExDefault()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Inline</h3>
        <p style(exampleLead)>
          Start-aligned row with brand-destructive border for partial failures.
        </p>
        <error-state variant="inline" style(stageCol)>
          <strong slot="title">Partial failure</strong>
          One pane could not refresh.
          <button slot="action" type="button" variant="outline" size="sm">Retry</button>
        </error-state>
        <code-block code={erExInline()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          <code>component.error-state.bg</code> · <code>border</code> · <code>fg</code>.
          Icon keeps brand destructive.
        </p>
        <error-state style(stageCol)>
          <strong slot="title">Something went wrong</strong>
          Rest chrome from component leaves.
        </error-state>
        <code-block code={erExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
