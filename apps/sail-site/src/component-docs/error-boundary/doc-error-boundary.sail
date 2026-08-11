<!--
  <doc-error-boundary> — first-class docs host for /components/error-boundary.
  Specimen shows healthy isolation; message knob documents fallback copy.
  Real throws are exercised by l1-conformance behavior — not the docs stage.
-->

<controller>
import type { Scope } from './doc-error-boundary.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const message = signal('This panel could not load.')
  const ebMeta = derived(() => {
    const m = message()
    return m
      ? `message set · healthy display:contents · component.error-boundary.*`
      : `default message · healthy display:contents · component.error-boundary.*`
  })
  const entry = DETAILS['error-boundary']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const ebExWrap = () =>
    `<error-boundary message="This panel could not load.">\n  <panel />\n</error-boundary>`
  const ebExEvents = () =>
    `// onErrorCaught → telemetry; onErrorReset → after Retry / el.reset()\n<error-boundary\n  message="Could not render chart"\n  onErrorCaught={onCaught}\n  onErrorReset={onReset}\n>\n  <chart />\n</error-boundary>`
  const ebExRest = () =>
    `<error-boundary>\n  <!-- healthy root is display:contents — zero layout impact -->\n  <p>Child tree isolated from render faults.</p>\n</error-boundary>`
  return {
    message,
    ebMeta,
    heading,
    blurb,
    props,
    usageCode,
    ebExWrap,
    ebExEvents,
    ebExRest,
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
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style faceRow = { box: w(full) }
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
            <div style(centerBox)>
              <error-boundary message={message()}>
                <p style(mutedSm)>Child tree isolated from render faults. Healthy boundary is display:contents — no layout box.</p>
              </error-boundary>
            </div>
            <p style(monoXs)>{ebMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Message</label>
            <input
              type="text"
              bind:value={message}
              size="sm"
              placeholder="user-facing fallback copy"
              style(fullW)
            />
            <p style(mutedXs)>
              What the user reads in the fallback — never the raw Error string.
              Thrown faults are exercised by l1-conformance behavior, not this stage.
            </p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Events</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>onErrorCaught</code> — telemetry (detail.error / source)</p>
              <p style(mutedXs)><code>onErrorReset</code> — after Retry / <code>el.reset()</code></p>
            </column>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <column gap="1" style(faceRow)>
              <p style(mutedXs)><code>component.error-boundary.bg</code> · border · fg</p>
              <p style(mutedXs)><code>icon-fg</code> · <code>message-fg</code></p>
              <p style(mutedXs)><code>retry-bg</code> · <code>retry-fg</code></p>
            </column>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Wrap a risky subtree</h3>
        <p style(exampleLead)>
          A throw inside swaps to the built-in fallback (or <code>slot="fallback"</code>).
          Retry brings content back.
        </p>
        <error-boundary message="This panel could not load.">
          <p style(mutedSm)>Healthy content — boundary invisible.</p>
        </error-boundary>
        <code-block code={ebExWrap()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Telemetry hooks</h3>
        <p style(exampleLead)>
          <code>onErrorCaught</code> carries the real error out-of-band;
          <code>onErrorReset</code> fires when the user recovers.
        </p>
        <code-block code={ebExEvents()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Rest tokens (themeable)</h3>
        <p style(exampleLead)>
          Fallback chrome from <code>component.error-boundary.*</code> leaves.
          Healthy root does not paint.
        </p>
        <error-boundary>
          <p style(mutedSm)>Child tree isolated from render faults.</p>
        </error-boundary>
        <code-block code={ebExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
