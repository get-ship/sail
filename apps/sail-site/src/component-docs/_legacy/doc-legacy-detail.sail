<!--
  <doc-legacy-detail> — residual docs host for packages without a first-class
  doc-<slug> host yet.

  First-class (do NOT re-add stages here): button, badge, fieldset, chip, alert,
  separator, spinner, skeleton, icon, progress, meter, label, field, radio,
  radio-group, switch, slider, input, textarea, select, scroll-area, ticker,
  table, pagination, toggle-group, tabs, navbar, card, avatar, breadcrumb,
  dialog, popover, tooltip, dropdown, hover-card, context-menu, toast,
  accordion, combobox, theme-picker.
  Peel remaining tags into component-docs/<slug>/doc-<slug>.sail over time.
  checkbox stays alias residual (no CE invent — demos <checkbox >).

  Specimens are intentional live demos (hub-quality), not random float.
  Props/usage come from DETAILS + pack meta. Viewer variant chips drive tags
  that still expose a variantAttr in DETAILS.
-->

<scope>
  props: {
    docTag = ''
  }

  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageColMd = { box: w(panel-max-w-md).maxW(full) }
  style fullW = { box: w(full) }
  style mediaStage = { box: w(full).maxW(panel-max-w-sm).h(48).minH(48).overflow(hidden), paint: radius(md) }
  style fullMedia = { box: w(full).h(full) }
  style ticker = { box: w(full).maxW(panel-max-w-md).overflow(hidden).padY(3), paint: bg(muted).radius(md) }
  style tickerText = { box: padX(4), paint: fg(muted-foreground), text: fontSize(sm).fontFamily(mono).whitespace(nowrap) }
  style scrollDemo = { box: maxH(panel-max-h-sm).w(full).maxW(panel-max-w-sm).pad(3), paint: borderWidth(1).border(border-subtle).radius(md) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }
  style pressBtn = { animate: press({ scale: 'press' }) }
  style toastStage = { box: w(full).maxW(panel-max-w-sm) }
  style sepVRow = { box: h(10) }
  style note = { paint: fg(muted-foreground), text: fontSize(xs) }
</scope>

<controller>
import type { Scope } from './doc-legacy-detail.sail'
import { DETAILS, type DetailEntry } from '../../compositions/component-details'
import {
  PACK_PROPS,
  PACK_TAGS,
  packBlurbFor,
  packLabelFor,
  packSlotCopy,
} from '../../compositions/component-pack-meta'
import { docMeta } from '../../compositions/component-docs-registry'

export default controller((ctx: Scope) => {
  const tag = derived(() => {
    const fromProp = ctx.docTag?.() ?? ''
    if (fromProp) return String(fromProp)
    return ''
  })

  const meta = derived(() => docMeta(tag()))
  const entry = derived<DetailEntry | null>(() => DETAILS[tag()] ?? null)
  const knownPack = derived(() => PACK_TAGS.has(tag()))

  const variant = signal('')
  effect(() => {
    const e = entry()
    const v = e?.variants?.[0] ?? ''
    variant.set(v)
  })
  const onVariantChange = (detail: { variant?: string; value?: string } | string): void => {
    const v =
      typeof detail === 'string' ? detail : (detail?.variant ?? detail?.value ?? '')
    variant.set(v)
  }

  // Interactive demo state shared across residual specimens.
  const agreed = signal(true)
  const presses = signal(0)
  const press = (): void => presses.set((n) => n + 1)
  // Pack media src used to be set here via requestAnimationFrame + document.
  // That is DOM work — Pure forbids it. Residual media tiles stay without
  // the injected DEMO_PHOTO until a use: action owns this.

  const heading = () => meta()?.heading ?? entry()?.heading ?? tag()
  const blurb = () => meta()?.blurb ?? entry()?.blurb ?? packBlurbFor(tag())
  const props = () => entry()?.props ?? []
  const usageCode = () => entry()?.usage ?? ''
  const packBlurb = () => packBlurbFor(tag())
  const packProps = () => PACK_PROPS[tag()] ?? []
  const packLabel = () => packLabelFor(tag())
  const slotCopy = () => (knownPack() ? packSlotCopy(tag()) : '')
  // Hide chips when package meta says stage owns knobs (first-class) or empty variants.
  const stageOwns = () => meta()?.stageOwnsKnobs === true
  const viewerVariants = () => (stageOwns() ? [] : entry()?.variants ?? [])
  const viewerVariantAttr = () => (stageOwns() ? '' : entry()?.variantAttr ?? '')

  // Per-tag stage flags — keep separate <if> blocks (no mega nested soup).
  const is = (slug: string) => () => tag() === slug
  const stage = {
    // Residual peel → accordion/combobox/theme-picker first-class
    // B6 peeled → tabs/navbar/card/avatar/breadcrumb/dialog/popover/tooltip/dropdown/hover-card/context-menu/toast
    // B5 peeled → scroll-area/ticker/table/pagination/toggle-group
    // B4 peeled → a/kbd/prose/code-block/empty-state/error-state/error-boundary
    // B3 peeled → input/textarea/select; checkbox alias residual
    checkbox: is('checkbox'),
    // B2 peeled → label/field/radio/radio-group/switch/slider
    dropzone: is('dropzone'),
    // B1 peeled → alert/chip/separator/spinner/skeleton/icon/progress/meter
    layout: is('layout'),
    // packs
    sailAreaChart: is('sail-area-chart'),
    sailBarChart: is('sail-bar-chart'),
    sailDonut: is('sail-donut'),
    sailGauge: is('sail-gauge'),
    sailHeatmap: is('sail-heatmap'),
    sailLineChart: is('sail-line-chart'),
    sailSparkline: is('sail-sparkline'),
    sailMediaImage: is('sail-media-image'),
    sailMediaAv: is('sail-media-av'),
    sailMessage: is('sail-message'),
    sailCitation: is('sail-citation'),
    sailComposer: is('sail-composer'),
    sailThinkingDots: is('sail-thinking-dots'),
    sailToolCall: is('sail-tool-call'),
    sailMermaid: is('sail-mermaid'),
    sailCodeEditor: is('sail-code-editor'),
  }

  return {
    tag,
    heading,
    blurb,
    props,
    usageCode,
    knownPack,
    packBlurb,
    packProps,
    packLabel,
    slotCopy,
    viewerVariants,
    viewerVariantAttr,
    variant,
    onVariantChange,
    stage,
    agreed,
    presses,
    press,
  }
})
</controller>

<column gap="4" style(page)>
  <sail-component-viewer
    name={heading()}
    blurb={knownPack() ? packBlurb() : blurb()}
    pack={knownPack() ? packLabel() : 'Component primitives'}
    variant-attr={viewerVariantAttr()}
    usage={usageCode()}
    variants={viewerVariants()}
    rows={knownPack() ? packProps() : props()}
    on-variant-change={onVariantChange}
  >
    <column slot="stage" gap="4" align="stretch" style({ box: w(full).minW(0) })>
      <if {stage.checkbox()}>
        <column gap="2" align="start" style(stageCol)>
          <label style(mutedSm)>
            <checkbox bind:checked={agreed} /> Email me product updates
          </label>
          <p style(note)>Catalog alias — <code>input type=checkbox</code>, not a separate tag.</p>
        </column>
      </if>
      <if {stage.dropzone()}>
        <card dropzone style({ box: pad(6).w(full).maxW(panel-max-w-sm) })>
          Drop a file, or click to browse
        </card>
        <p style(note)>Action / attribute — not an L1 host tag.</p>
      </if>
      <if {stage.layout()}>
        <column gap="3" style(stageColMd)>
          <row gap="2">
            <card style({ box: pad(3).grow(1) })>row</card>
            <card style({ box: pad(3).grow(1) })>gap</card>
          </row>
          <grid cols="2" gap="2">
            <card style({ box: pad(3) })>grid</card>
            <card style({ box: pad(3) })>cols=2</card>
          </grid>
          <center style(centerBox)>center</center>
          <p style(note)>Keywords — not a SOURCES host. row · column · grid · center.</p>
        </column>
      </if>
      <!-- Pack specimens (when pack tag is known) -->
      <if {stage.sailAreaChart()}>
        <sail-area-chart style(stageColMd)></sail-area-chart>
      </if>
      <if {stage.sailBarChart()}>
        <sail-bar-chart style(stageColMd)></sail-bar-chart>
      </if>
      <if {stage.sailDonut()}>
        <sail-donut></sail-donut>
      </if>
      <if {stage.sailGauge()}>
        <sail-gauge></sail-gauge>
      </if>
      <if {stage.sailHeatmap()}>
        <sail-heatmap style(stageColMd)></sail-heatmap>
      </if>
      <if {stage.sailLineChart()}>
        <sail-line-chart style(stageColMd)></sail-line-chart>
      </if>
      <if {stage.sailSparkline()}>
        <sail-sparkline></sail-sparkline>
      </if>
      <if {stage.sailMediaImage()}>
        <div style(mediaStage)>
          <sail-media-image style(fullMedia)></sail-media-image>
        </div>
      </if>
      <if {stage.sailMediaAv()}>
        <sail-media-av style(stageColMd)></sail-media-av>
      </if>
      <if {stage.sailMessage()}>
        <sail-message style(stageColMd)></sail-message>
      </if>
      <if {stage.sailCitation()}>
        <sail-citation></sail-citation>
      </if>
      <if {stage.sailComposer()}>
        <sail-composer style(stageColMd)></sail-composer>
      </if>
      <if {stage.sailThinkingDots()}>
        <sail-thinking-dots></sail-thinking-dots>
      </if>
      <if {stage.sailToolCall()}>
        <sail-tool-call></sail-tool-call>
      </if>
      <if {stage.sailMermaid()}>
        <sail-mermaid style(stageColMd)></sail-mermaid>
      </if>
      <if {stage.sailCodeEditor()}>
        <sail-code-editor style(stageColMd)></sail-code-editor>
      </if>

      <if {slotCopy()}>
        <p style(monoXs)>{slotCopy()}</p>
      </if>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
