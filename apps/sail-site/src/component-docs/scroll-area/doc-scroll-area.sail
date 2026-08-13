<!--
  <doc-scroll-area> — /components/scroll-area
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style specimen = { box: size({ w: full, max-w: panel-max-w-md }).space({ pad: 2 }).overflow({ all: visible }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style mutedSm = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style mutedXs = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
  style scrollDemoV = {
    box: size({ w: full, h: 32, max-w: panel-max-w-sm }).space({ pad: 3 }),
    paint: border({ color: border-subtle, width: 1 }).radius(md).fill({ color: surface-0 }),
}
  style scrollDemoH = {
    box: size({ w: full, max-w: panel-max-w-sm }).space({ pad: 3 }),
    paint: border({ color: border-subtle, width: 1 }).radius(md).fill({ color: surface-0 }),
}
  style wideBand = { box: size({ w: panel-max-w-lg }), text: type({ wrap: nowrap }) }
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
      return `<scroll-area orientation="horizontal" style({ box: size({ w: full }).space({ pad: 3 }), paint: border({ color: border, width: 1 }).radius(md) })>
  <p style({ text: type({ wrap: nowrap }) })>Wide content that overflows →</p>
</scroll-area>`
    }
    if (o === 'both') {
      return `<scroll-area orientation="both" style({ box: size({ w: full, h: 32 }).space({ pad: 3 }), paint: border({ color: border, width: 1 }).radius(md) })>
  <p>Overflow content…</p>
</scroll-area>`
    }
    return `<scroll-area orientation="vertical" style({ box: size({ w: full, h: 32 }).space({ pad: 3 }), paint: border({ color: border, width: 1 }).radius(md) })>
  <p>Overflow content…</p>
</scroll-area>`
  })

  const saExVertical = () =>
    `<scroll-area orientation="vertical" style({ box: size({ w: full, h: 32 }).space({ pad: 3 }), paint: border({ color: border, width: 1 }).radius(md) })>
  <p>Overflow content…</p>
</scroll-area>`
  const saExHorizontal = () =>
    `<scroll-area orientation="horizontal" style({ box: size({ w: full }).space({ pad: 3 }), paint: border({ color: border, width: 1 }).radius(md) })>
  <p style({ text: type({ wrap: nowrap }) })>Wide content that overflows →</p>
</scroll-area>`
  const saExBoth = () =>
    `<scroll-area orientation="both" style({ box: size({ w: full, h: 32 }).space({ pad: 3 }), paint: border({ color: border, width: 1 }).radius(md) })>
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
