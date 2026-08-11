<!--
  <doc-ticker> — first-class docs host for /components/ticker.
  Specimen left, knobs right — duration × reverse. No component.ticker.* faces.
-->

<controller>
import type { Scope } from './doc-ticker.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const duration = signal('normal')
  const reverse = signal(false)
  const tkMeta = derived(() => {
    const dir = reverse() ? 'reverse (+50%)' : 'forward (−50%)'
    return `${duration()} · ${dir} · author paint · no component.ticker.*`
  })
  const entry = DETAILS.ticker
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const tkExNormal = () =>
    `<ticker duration="normal" style({ box: w(full).overflow(hidden).padY(2), paint: bg(muted).radius(md) })>\n  <p>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>\n</ticker>`
  const tkExReverse = () =>
    `<ticker duration="slow" reverse>\n  <p>rightward marquee…&nbsp;</p>\n</ticker>`
  const tkExFast = () =>
    `<ticker duration="fast">\n  <p>fast 16s loop…&nbsp;</p>\n</ticker>`
  return {
    duration,
    reverse,
    tkMeta,
    heading,
    blurb,
    props,
    usageCode,
    tkExNormal,
    tkExReverse,
    tkExFast,
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
  style ticker = {
    box: w(full).maxW(panel-max-w-md).overflow(hidden).padY(3),
    paint: bg(muted).radius(md),
  }
  style tickerText = {
    box: padX(4),
    paint: fg(muted-foreground),
    text: fontSize(sm).fontFamily(mono).whitespace(nowrap),
  }
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
            <if {reverse()}>
              <ticker duration={duration()} reverse style(ticker)>
                <p style(tickerText)>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
              </ticker>
            {else =>
              <ticker duration={duration()} style(ticker)>
                <p style(tickerText)>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
              </ticker>}</if>
            <p style(monoXs)>{tkMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Duration</label>
            <select size="sm" bind:value={duration} label="Duration">
              <option value="fast">fast</option>
              <option value="normal">normal</option>
              <option value="slow">slow</option>
            </select>
            <p style(mutedXs)>Loop length for use:marquee — fast 16s · normal 24s · slow 32s.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Reverse</label>
            <row gap="2" align="center">
              <switch bind:checked={reverse} size="sm" tone="primary" label="Reverse"></switch>
              <span style(sizeValue)>{reverse() ? 'rightward' : 'leftward'}</span>
            </row>
            <p style(mutedXs)>Presence attr — flip translate to +50%.</p>
          </column>
          <column gap="2">
            <label style(knobLabel)>Faces</label>
            <p style(mutedXs)>
              None — no <code>component.ticker.*</code> leaves. Author paint on the host
              (muted band here is stage Style only).
            </p>
          </column>
        </column>
      </row>
    </column>

    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>

      <column gap="2">
        <h3 style(exampleH)>Default marquee</h3>
        <p style(exampleLead)>
          Host clips; <code>use:marquee</code> dual-copies the slot and loops translate-x.
        </p>
        <ticker duration="normal" style(ticker)>
          <p style(tickerText)>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
        </ticker>
        <code-block code={tkExNormal()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Reverse + slow</h3>
        <p style(exampleLead)>
          <code>reverse</code> walks the band the other way; <code>duration="slow"</code> lengthens the loop.
        </p>
        <ticker duration="slow" reverse style(ticker)>
          <p style(tickerText)>rightward marquee · reduced motion settles ·&nbsp;</p>
        </ticker>
        <code-block code={tkExReverse()}></code-block>
      </column>

      <column gap="2">
        <h3 style(exampleH)>Fast</h3>
        <p style(exampleLead)><code>duration="fast"</code> — 16s loop.</p>
        <ticker duration="fast" style(ticker)>
          <p style(tickerText)>fast loop · paint is yours ·&nbsp;</p>
        </ticker>
        <code-block code={tkExFast()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
