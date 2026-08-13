<!--
  <doc-skeleton> — /components/skeleton
  Shared chrome: split + snippet copy + usage Code|Live|Split.
-->

<scope>
  style page = { box: size({ w: full, max-w: breakpoint-desktop }).space({ mx: auto }).overflow({ all: visible }) }
  style specimen = { box: size({ w: full, max-w: panel-max-w-sm }).space({ pad: 2 }).overflow({ all: visible }) }
  style fullW = { box: size({ w: full }) }
  style grow1 = { box: flow({ grow: 1 }) }
  style skelBlock = { box: size({ w: full, max-w: panel-max-w-sm }) }
  style knobLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, weight: medium, tracking: wide, case: uppercase }),
}
  style sizeValue = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style tickRow = { box: size({ w: full }).space({ pad-x: 1_5, pad-t: 1 }) }
  style tick = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style exampleBlock = { box: size({ w: full }).space({ pad-t: 2, pad-b: 4 }).overflow({ all: visible }) }
  style sectionLabel = {
    paint: ink({ color: muted-foreground }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
}
</scope>

<controller>
import type { Scope } from './doc-skeleton.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {
  const SKEL_SIZES = ['sm', 'md', 'lg'] as const
  const shape = signal('text')
  const sizeIdx = signal(1)

  const skelSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SKEL_SIZES.length - 1)
    return SKEL_SIZES[i] ?? 'md'
  })

  const entry = DETAILS.skeleton
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage

  const specimenCode = derived(() => {
    const sh = shape() || 'text'
    const a: string[] = [`shape="${sh}"`]
    if (sh !== 'block' && skelSize() !== 'md') a.push(`size="${skelSize()}"`)
    return `<skeleton ${a.join(' ')}></skeleton>`
  })

  const skelExLines = () =>
    `<column gap="2">\n  <skeleton shape="text" size="md"></skeleton>\n  <skeleton shape="text" size="sm"></skeleton>\n  <skeleton shape="line" size="sm"></skeleton>\n</column>`
  const skelExAvatar = () =>
    `<row gap="3" align="center">\n  <skeleton shape="circle" size="md"></skeleton>\n  <column gap="2">\n    <skeleton shape="text" size="sm"></skeleton>\n    <skeleton shape="text" size="sm"></skeleton>\n  </column>\n</row>`
  const skelExBlock = () =>
    `<skeleton shape="block"></skeleton>\n<skeleton shape="text" size="md"></skeleton>`

  return {
    shape,
    sizeIdx,
    skelSize,
    heading,
    blurb,
    props,
    usageCode,
    specimenCode,
    skelExLines,
    skelExAvatar,
    skelExBlock,
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
      <if {shape() === 'circle'}>
        <skeleton shape="circle" size={skelSize()}></skeleton>
      {else-if {shape() === 'block'} =>
        <skeleton shape="block" style(skelBlock)></skeleton>
      }{else =>
        <column gap="2" style(fullW)>
          <skeleton shape={shape() || 'text'} size={skelSize()}></skeleton>
        </column>}</if>
    </column>

    <column slot="knobs" gap="4" align="stretch">
      <column gap="2">
        <label style(knobLabel)>Shape</label>
        <select size="sm" bind:value={shape} label="Shape">
          <option value="text">text</option>
          <option value="line">line</option>
          <option value="block">block</option>
          <option value="circle">circle</option>
        </select>
      </column>
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <label style(knobLabel)>Size</label>
          <span style(sizeValue)>{skelSize()}</span>
        </row>
        <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size" ticks="sm,md,lg"></slider>
      </column>
    </column>

    <column slot="usage-preview" gap="3" style(specimen)>
      <column gap="2" style(fullW)>
        <skeleton shape="text" size="md"></skeleton>
        <skeleton shape="text" size="sm"></skeleton>
        <skeleton shape="line" size="sm"></skeleton>
      </column>
    </column>

    <column gap="2" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <doc-example
        title="Text lines"
        lead='Stack shape="text" / line bars for a paragraph placeholder.'
        code={skelExLines()}
      >
        <column gap="2" style(specimen)>
          <skeleton shape="text" size="md"></skeleton>
          <skeleton shape="text" size="sm"></skeleton>
          <skeleton shape="line" size="sm"></skeleton>
        </column>
      </doc-example>
      <doc-example
        title="Avatar row"
        lead="Compose circle with text bars for a list-item loading state."
        code={skelExAvatar()}
      >
        <row gap="3" align="center" style(specimen)>
          <skeleton shape="circle" size="md"></skeleton>
          <column gap="2" style(grow1)>
            <skeleton shape="text" size="sm"></skeleton>
            <skeleton shape="text" size="sm"></skeleton>
          </column>
        </row>
      </doc-example>
      <doc-example
        title="Block + text"
        lead="Block tiles ignore size; text bars scale with size."
        code={skelExBlock()}
      >
        <column gap="2" style(specimen)>
          <skeleton shape="block"></skeleton>
          <skeleton shape="text" size="md"></skeleton>
        </column>
      </doc-example>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
