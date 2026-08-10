<!--
  <doc-skeleton> — first-class docs host for /components/skeleton.
  Specimen left, knobs right (Mantine-style configurator) + examples walkthrough.
-->

<controller>
import type { Scope } from './doc-skeleton.sail'
import { DETAILS } from '../../compositions/component-details'

export default controller((_ctx: Scope) => {

  const SKEL_SIZES = ['sm', 'md', 'lg'] as const
  const variant = signal('text')
  const sizeIdx = signal(1)
  const skelSize = derived(() => {
    const i = Math.min(Math.max(0, Math.round(Number(sizeIdx()) || 0)), SKEL_SIZES.length - 1)
    return SKEL_SIZES[i] ?? 'md'
  })
  const skelMeta = derived(() => {
    const sh = variant() || 'text'
    const sizeNote = sh === 'block' ? 'size n/a (block)' : skelSize()
    return `${sh} · ${sizeNote} · sheen component.skeleton.*`
  })
  const entry = DETAILS.skeleton
  const heading = () => entry.heading
  const blurb = () => entry.blurb
  const props = () => entry.props
  const usageCode = () => entry.usage
  const skelExLines = () =>
    `<column gap="2">\n  <skeleton shape="text" size="md"></skeleton>\n  <skeleton shape="text" size="sm"></skeleton>\n  <skeleton shape="line" size="sm"></skeleton>\n</column>`
  const skelExAvatar = () =>
    `<row gap="3" align="center">\n  <skeleton shape="circle" size="md"></skeleton>\n  <column gap="2">\n    <skeleton shape="text" size="sm"></skeleton>\n    <skeleton shape="text" size="sm"></skeleton>\n  </column>\n</row>`
  const skelExRest = () =>
    `<skeleton shape="block"></skeleton>\n<!-- sheen → component.skeleton.bg / highlight -->`
  return {
    variant, sizeIdx, skelSize, skelMeta,
    heading, blurb, props, usageCode, skelExLines, skelExAvatar, skelExRest,
  }

})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageColMd = { box: w(panel-max-w-md).maxW(full) }
  style meterCol = { box: w(panel-max-w-sm) }
  style fullW = { box: w(full) }
  style grow1 = { box: grow(1) }
  style stageSplit = { box: w(full).minW(0).minH(64) }
  style stageCanvas = { box: grow(1).basis(0).minW(0).minH(64).pad(8) }
  style knobPanel = {
    box: w(panel-max-w-sm).shrink(0).pad(5).minH(64),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style knobLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).fontWeight(medium).tracking(wide).textCase(uppercase),
  }
  style sizeValue = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style swatchRow = { box: w(full) }
  style tickRow = { box: w(full).padT(1) }
  style tick = {
    box: grow(1).basis(0).minW(0),
    paint: fg(muted-foreground),
    text: fontSize(xs).fontFamily(mono).textAlign(center),
  }
  style swatch = {
    box: w(7).h(7).shrink(0),
    paint: radius(full).cursor(pointer).borderWidth(2).border(transparent).shadow(xs),
    when: {
      selected: { paint: border(foreground).shadow(sm) },
      hover: { paint: opacity(90) },
    },
  }
  style swatchPrimary = { paint: bg(primary) }
  style swatchSuccess = { paint: bg(success) }
  style swatchWarning = { paint: bg(warning) }
  style swatchDestructive = { paint: bg(destructive) }
  style swatchAccent = { paint: bg(accent) }
  style swatchMuted = { paint: bg(muted).border(border) }
  style swatchInfo = { paint: bg(info) }
  style swatchRest = { paint: bg(background).border(border) }
  style exampleBlock = { box: w(full).padY(2) }
  style exampleH = { text: fontSize(md).fontWeight(semibold) }
  style exampleLead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style sectionLabel = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style sepVRow = { box: h(10) }
  style sepStageCol = { box: w(full).maxW(panel-max-w-sm) }
  style skelStageCol = { box: w(full).maxW(panel-max-w-sm) }
  style skelBlock = { box: w(full).maxW(panel-max-w-sm) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }
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
          <column gap="4" align="center" style(skelStageCol)>
            <if {variant() === 'circle'}>
              <skeleton shape="circle" size={skelSize()}></skeleton>
            {else-if {variant() === 'block'} =>
              <skeleton shape="block" style(skelBlock)></skeleton>
            }{else =>
              <column gap="2" style(fullW)>
                <skeleton shape={variant() || 'text'} size={skelSize()}></skeleton>
              </column>}</if>
            <p style(monoXs)>{skelMeta()}</p>
          </column>
        </center>
        <column gap="5" style(knobPanel)>
          <column gap="2">
            <label style(knobLabel)>Shape</label>
            <select size="sm" bind:value={variant} label="Shape">
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
            <slider bind:value={sizeIdx} min="0" max="2" step="1" size="sm" label="Size"></slider>
            <row gap="0" align="center" style(tickRow)>
              <span style(tick)>sm</span><span style(tick)>md</span><span style(tick)>lg</span>
            </row>
            <p style(mutedXs)>Size scales text/line height and circle diameter. Block ignores size.</p>
          </column>
        </column>
      </row>
    </column>
    <column gap="5" style(exampleBlock)>
      <p style(sectionLabel)>Examples</p>
      <column gap="2">
        <h3 style(exampleH)>Text lines</h3>
        <p style(exampleLead)>Stack <code>shape="text"</code> / <code>line</code> bars for a paragraph placeholder.</p>
        <column gap="2" style(skelStageCol)>
          <skeleton shape="text" size="md"></skeleton>
          <skeleton shape="text" size="sm"></skeleton>
          <skeleton shape="line" size="sm"></skeleton>
        </column>
        <code-block code={skelExLines()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Avatar row</h3>
        <p style(exampleLead)>Compose <code>circle</code> with text bars for a list-item loading state.</p>
        <row gap="3" align="center" style(skelStageCol)>
          <skeleton shape="circle" size="md"></skeleton>
          <column gap="2" style(grow1)>
            <skeleton shape="text" size="sm"></skeleton>
            <skeleton shape="text" size="sm"></skeleton>
          </column>
        </row>
        <code-block code={skelExAvatar()}></code-block>
      </column>
      <column gap="2">
        <h3 style(exampleH)>Sheen tokens (themeable)</h3>
        <p style(exampleLead)>Band colours ride <code>component.skeleton.bg</code> / <code>highlight</code>.</p>
        <column gap="2" style(skelStageCol)>
          <skeleton shape="block"></skeleton>
          <skeleton shape="text" size="md"></skeleton>
        </column>
        <code-block code={skelExRest()}></code-block>
      </column>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
