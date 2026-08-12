<!--
  <doc-scroll-area> — /components/scroll-area
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full).overflow(visible) }
  style specimen = { box: w(full).maxW(panel-max-w-md).pad(2).overflow(visible) }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style exampleBlock = { box: w(full).padT(2).padB(4).overflow(visible) }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
  style scrollDemoV = {
    box: h(32).w(full).maxW(panel-max-w-sm).pad(3),
    paint: borderWidth(1).border(border-subtle).radius(md).bg(surface-0),
  }
  style scrollDemoH = {
    box: w(full).maxW(panel-max-w-sm).pad(3),
    paint: borderWidth(1).border(border-subtle).radius(md).bg(surface-0),
  }
  style wideBand = { box: w(panel-max-w-lg), text: whitespace(nowrap) }
</scope>

<controller>
import type { Scope } from './doc-scroll-area.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const orientation = signal('vertical')

  const entry = DETAILS['scroll-area']
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const o = orientation()
    if (o === 'horizontal') {
      return `<scroll-area orientation="horizontal" style({ box: w(full).pad(3), paint: borderWidth(1).border(border).radius(md) })>
  <p style({ text: whitespace(nowrap) })>Wide content that overflows →</p>
</scroll-area>`
    }
    if (o === 'both') {
      return `<scroll-area orientation="both" style({ box: h(32).w(full).pad(3), paint: borderWidth(1).border(border).radius(md) })>
  <p>Overflow content…</p>
</scroll-area>`
    }
    return `<scroll-area orientation="vertical" style({ box: h(32).w(full).pad(3), paint: borderWidth(1).border(border).radius(md) })>
  <p>Overflow content…</p>
</scroll-area>`
  })

  const saExVertical = () =>
    `<scroll-area orientation="vertical" style({ box: h(32).w(full).pad(3), paint: borderWidth(1).border(border).radius(md) })>
  <p>Overflow content…</p>
</scroll-area>`
  const saExHorizontal = () =>
    `<scroll-area orientation="horizontal" style({ box: w(full).pad(3), paint: borderWidth(1).border(border).radius(md) })>
  <p style({ text: whitespace(nowrap) })>Wide content that overflows →</p>
</scroll-area>`
  const saExBoth = () =>
    `<scroll-area orientation="both" style({ box: h(32).w(full).pad(3), paint: borderWidth(1).border(border).radius(md) })>
  <p>Tall and wide overflow…</p>
</scroll-area>`

  return {
    orientation,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    saExVertical,
    saExHorizontal,
    saExBoth,
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
      <if {orientation() === 'horizontal'}>
        <scroll-area orientation="horizontal" style(scrollDemoH)>
          <p style(mutedSm, wideBand)>
            Wide band — tokens · signals · a11y · hot reload · SSR · layout · overflow →
          </p>
        </scroll-area>
      {else =>
        <if {orientation() === 'both'}>
          <scroll-area orientation="both" style(scrollDemoV)>
            <p style(mutedSm)>Line 1 — overflow both axes</p>
            <p style(mutedSm, wideBand)>Line 2 — wide content that needs horizontal scroll as well →</p>
            <p style(mutedSm)>Line 3</p>
            <p style(mutedSm)>Line 4</p>
            <p style(mutedSm)>Line 5</p>
            <p style(mutedSm)>Line 6</p>
            <p style(mutedSm)>Line 7</p>
            <p style(mutedSm)>Line 8</p>
          </scroll-area>
        {else =>
          <scroll-area orientation="vertical" style(scrollDemoV)>
            <p style(mutedSm)>Line 1 — overflow content</p>
            <p style(mutedSm)>Line 2</p>
            <p style(mutedSm)>Line 3</p>
            <p style(mutedSm)>Line 4</p>
            <p style(mutedSm)>Line 5</p>
            <p style(mutedSm)>Line 6</p>
            <p style(mutedSm)>Line 7</p>
            <p style(mutedSm)>Line 8</p>
          </scroll-area>}</if>}</if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Orientation</label>
        <select size="sm" bind:value={orientation} label="Orientation">
          <option value="vertical">vertical</option>
          <option value="horizontal">horizontal</option>
          <option value="both">both</option>
        </select>
        <p style(mutedXs)>
          vertical scrolls down, horizontal scrolls sideways, both allows either.
          Constrain the host size so content can overflow.
        </p>
      </column>
    </column>

    <column slot="usage-preview" gap="3" align="stretch" style(specimen)>
      <scroll-area orientation="vertical" style(scrollDemoV)>
        <p style(mutedSm)>Line 1</p>
        <p style(mutedSm)>Line 2</p>
        <p style(mutedSm)>Line 3</p>
        <p style(mutedSm)>Line 4</p>
        <p style(mutedSm)>Line 5</p>
        <p style(mutedSm)>Line 6</p>
      </scroll-area>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Vertical overflow"
        lead="Default orientation. Fix height so rows spill and the scrollbar appears."
        code={saExVertical()}
      >
        <scroll-area orientation="vertical" style(scrollDemoV)>
          <p style(mutedSm)>Line 1</p>
          <p style(mutedSm)>Line 2</p>
          <p style(mutedSm)>Line 3</p>
          <p style(mutedSm)>Line 4</p>
          <p style(mutedSm)>Line 5</p>
          <p style(mutedSm)>Line 6</p>
        </scroll-area>
      </doc-example>
      <doc-example
        title="Horizontal overflow"
        lead='orientation="horizontal" — a nowrap band wider than the host.'
        code={saExHorizontal()}
      >
        <scroll-area orientation="horizontal" style(scrollDemoH)>
          <p style(mutedSm, wideBand)>Wide content that overflows horizontally →</p>
        </scroll-area>
      </doc-example>
      <doc-example
        title="Both axes"
        lead='orientation="both" — tall and wide content can scroll either way.'
        code={saExBoth()}
      >
        <scroll-area orientation="both" style(scrollDemoV)>
          <p style(mutedSm)>Line 1 — overflow both axes</p>
          <p style(mutedSm, wideBand)>Line 2 — wide content that needs horizontal scroll as well →</p>
          <p style(mutedSm)>Line 3</p>
          <p style(mutedSm)>Line 4</p>
          <p style(mutedSm)>Line 5</p>
          <p style(mutedSm)>Line 6</p>
        </scroll-area>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
