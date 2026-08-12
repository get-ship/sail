<!--
  <sail-component-viewer> — shared docs/detail chrome. Declarative only.

  layout="split" (SSOT configurator):
    row
      left  grow · center both axes → slot name=specimen
             optional snippet prop → live markup + copy under specimen
      right fixed knobs column · start-aligned → slot name=knobs

  layout="stage" (legacy / host-owned row):
    full-bleed → slot name=stage

  Usage: code (default) · live (slot usage-preview) · split (side by side).

  Hosts own specimen state. Variant chips call onVariantChange — they never
  walk the DOM or stamp attributes onto slotted content.
-->

<scope>
  props: {
    /** Component name — the heading. */
    name = ''
    /** Short blurb under the heading. */
    blurb = ''
    /** Pack label chip (e.g. "L1 · language element"). */
    pack = ''
    /**
     * Configurator chrome:
     *   split — specimen | knobs (preferred)
     *   stage — host owns the full stage row in slot=stage
     */
    layout: 'split' | 'stage' = 'stage'
    /** Which attribute the variant chips label (empty = no chips). */
    'variant-attr' = ''
    /** Variant values for optional chips. Host binds the specimen itself. */
    variants: string[]
    /** Prop rows — registry shape (`props` is a runtime context key → `rows`). */
    rows: PropRow[]
    /** A complete, compiling usage program. */
    usage = ''
    /**
     * Live specimen markup (tracks host knobs). When set, viewer renders a
     * <code-block copy> under the specimen — hosts should not reimplement chrome.
     */
    snippet = ''
    /**
     * When true, Usage offers Code / Live / Split. Host must fill
     * slot="usage-preview" with a live render of the program.
     */
    'usage-live' = false
    /** Hide the a11y assurance chip. */
    'no-a11y-chip' = false
    /** Fired when a variant chip is picked. Host owns specimen binding. */
    onVariantChange: (detail: { variant: string }) => void
  }
  style sectionLabel = {
    paint: fg(muted-foreground),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
  style stageSplit = { box: w(full).minW(0).minH(64).overflow(visible) }
  /**
   * Specimen canvas. pad must clear focus ring (ring-width + ring-offset ≈ 5px;
   * use pad(4)+ so knobs and specimen never clip outlines on the frame edge).
   */
  style stageCanvas = {
    box: grow(1).basis(0).minW(0).minH(64).pad(6).overflow(visible),
  }
  style specimenStack = { box: w(full).maxW(full).pad(1).overflow(visible) }
  /**
   * Snippet frame wraps the bordered code-block. Copy is absolute *inside*
   * that frame (top-right of the pre box). Hover-reveal; pre padE reserves
   * room so glyphs never sit under the button.
   */
  style snippetShell = {
    box: w(full).maxW(full).position(relative),
    paint: group(),
  }
  style snippetCopy = {
    box: position(absolute).top(3).right(3).z(2),
    paint: bg(surface-0).fg(muted-foreground).radius(sm).opacity(0).pointerEvents(none).shadow(xs),
    when: {
      group-hover: { paint: opacity(100).pointerEvents(auto) },
    },
  }
  /** Knobs rail: space scale, never panel-* tokens. Pad for focus rings. */
  style knobColumn = {
    box: w(64).minW(64).shrink(0).pad(5).minH(64).overflow(visible),
    paint: borderLWidth(1).borderL(border-subtle),
  }
  style usageFrame = {
    box: w(full).pad(4).overflow(visible),
    paint: radius(lg).borderWidth(1).border(border).bg(surface-0),
  }
  style usagePreviewPad = { box: w(full).pad(4).overflow(visible) }
</scope>

<controller>

import type { Scope } from './sail-component-viewer.sail'

export interface PropRow {
  name: string
  type: string
  desc: string
}

export default controller(({ variants, rows, usage, snippet, usageLive, variantAttr, layout, onVariantChange }: Scope) => {
  const active = signal('')
  const copied = signal(false)
  const usageCopied = signal(false)
  let copiedTimer: ReturnType<typeof setTimeout> | null = null
  let usageCopiedTimer: ReturnType<typeof setTimeout> | null = null
  /** code | live | split — usage program surface. */
  const usageView = signal<'code' | 'live' | 'split'>('code')

  const variantList = derived(() => variants() ?? [])
  const hasVariants = derived(
    () => (variantAttr() ?? '') !== '' && variantList().length > 0,
  )
  const current = derived(() => active() || variantList()[0] || '')
  const propRows = derived(() => rows() ?? [])
  const hasProps = derived(() => propRows().length > 0)
  const hasUsage = derived(() => (usage() ?? '') !== '')
  const hasSnippet = derived(() => (snippet() ?? '').trim() !== '')
  const hasUsageLive = derived(() => !!usageLive())
  const isSplit = derived(() => layout() === 'split')
  const copyLabel = derived(() => (copied() ? 'Copied' : 'Copy'))
  const usageCopyLabel = derived(() => (usageCopied() ? 'Copied' : 'Copy'))
  const usageIsCode = derived(() => usageView() === 'code' || !usageLive())
  const usageIsLive = derived(() => usageLive() && usageView() === 'live')
  const usageIsSplit = derived(() => usageLive() && usageView() === 'split')
  const chips = derived(() =>
    variantList().map((v) => ({
      v,
      variant: v === current() ? 'solid' : 'ghost',
    })),
  )

  // Seed active to first variant when the list appears (no DOM writes).
  effect(() => {
    const list = variantList()
    if (list.length > 0 && !active()) active.set(list[0] ?? '')
  })

  const pickVariant = (v: string): void => {
    active.set(v)
    onVariantChange()?.({ variant: v })
  }

  const markCopied = (): void => {
    copied.set(true)
    if (copiedTimer) clearTimeout(copiedTimer)
    copiedTimer = setTimeout(() => copied.set(false), 1500)
  }
  const markUsageCopied = (): void => {
    usageCopied.set(true)
    if (usageCopiedTimer) clearTimeout(usageCopiedTimer)
    usageCopiedTimer = setTimeout(() => usageCopied.set(false), 1500)
  }
  const clipText = (): string => snippet() ?? ''
  const usageClipText = (): string => usage() ?? ''

  const showUsageCode = (): void => usageView.set('code')
  const showUsageLive = (): void => usageView.set('live')
  const showUsageSplit = (): void => usageView.set('split')

  return {
    current,
    chips,
    pickVariant,
    hasVariants,
    propRows,
    hasProps,
    hasUsage,
    hasSnippet,
    hasUsageLive,
    isSplit,
    copyLabel,
    usageCopyLabel,
    clipText,
    usageClipText,
    markCopied,
    markUsageCopied,
    usageIsCode,
    usageIsLive,
    usageIsSplit,
    showUsageCode,
    showUsageLive,
    showUsageSplit,
  }
})
</controller>

<column gap="4" style({ box: w(full).overflow(visible) })>
  <row gap="3" align="center">
    <h2 style({ text: fontFamily(mono).tracking(tight) })>{name()}</h2>
    <if {pack() !== ''}>
      <badge tone="primary" variant="soft">{pack()}</badge>
    </if>
    <if {!noA11yChip()}>
      <badge tone="muted" variant="outline">a11y ✓ · keyboard ✓</badge>
    </if>
  </row>
  <if {blurb() !== ''}>
    <p style({ paint: fg(muted-foreground) })>{blurb()}</p>
  </if>

  <column gap="0" style({ box: pad(0).overflow(visible), paint: radius(lg).borderWidth(1).border(border).bg(surface-0) })>
    <if {hasVariants()}>
      <row gap="2" align="center" style({ box: padX(5).padT(4).padB(2) })>
        <span style({ paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })>{variantAttr()}:</span>
        <for {chips() as c} key={c.v}>
          <button size="sm" tone="primary" variant={c.variant} on:click={() => pickVariant(c.v)}>{c.v}</button>
        </for>
      </row>
    </if>

    <if {isSplit()}>
      <row nowrap gap="0" align="stretch" style(stageSplit)>
        <center style(stageCanvas)>
          <column gap="4" align="center" style(specimenStack)>
            <slot name="specimen"></slot>
            <if {hasSnippet()}>
              <div style(snippetShell)>
                <button
                  size="sm"
                  variant="ghost"
                  tone="muted"
                  type="button"
                  use:clipboard={{ text: clipText, onCopy: markCopied }}
                  style(snippetCopy)
                >
                  {copyLabel()}
                </button>
                <code-block code={snippet()} lang="sail"></code-block>
              </div>
            </if>
          </column>
        </center>
        <column gap="5" align="start" style(knobColumn)>
          <slot name="knobs"></slot>
        </column>
      </row>
    </if>

    <if {!isSplit()}>
      <slot name="stage"></slot>
    </if>
  </column>

  <slot></slot>

  <if {hasUsage()}>
    <column gap="3" style({ box: padT(2).overflow(visible) })>
      <row gap="3" align="center" justify="between">
        <p style(sectionLabel)>Usage — a complete program</p>
        <if {hasUsageLive()}>
          <row gap="1" align="center">
            <button
              size="sm"
              tone="primary"
              variant={usageIsCode() ? 'solid' : 'ghost'}
              on:click={showUsageCode}
            >
              Code
            </button>
            <button
              size="sm"
              tone="primary"
              variant={usageIsLive() ? 'solid' : 'ghost'}
              on:click={showUsageLive}
            >
              Live
            </button>
            <button
              size="sm"
              tone="primary"
              variant={usageIsSplit() ? 'solid' : 'ghost'}
              on:click={showUsageSplit}
            >
              Split
            </button>
          </row>
        </if>
      </row>
      <if {usageIsCode()}>
        <div style(snippetShell)>
          <button
            size="sm"
            variant="ghost"
            tone="muted"
            type="button"
            use:clipboard={{ text: usageClipText, onCopy: markUsageCopied }}
            style(snippetCopy)
          >
            {usageCopyLabel()}
          </button>
          <code-block code={usage()} lang="sail"></code-block>
        </div>
      {else =>
        <row gap="4" align="start" style({ box: w(full).minW(0).overflow(visible) })>
          <if {usageIsSplit()}>
            <div style(snippetShell, { box: grow(1).basis(0).minW(0) })>
              <button
                size="sm"
                variant="ghost"
                tone="muted"
                type="button"
                use:clipboard={{ text: usageClipText, onCopy: markUsageCopied }}
                style(snippetCopy)
              >
                {usageCopyLabel()}
              </button>
              <code-block code={usage()} lang="sail"></code-block>
            </div>
          </if>
          <column gap="3" style(usageFrame, { box: grow(1).basis(0).minW(0) })>
            <slot name="usage-preview"></slot>
          </column>
        </row>}</if>
    </column>
  </if>

  <if {hasProps()}>
    <card style({ box: pad(4) })>
      <column gap="3">
        <p style(sectionLabel)>Props</p>
        <for {propRows() as r} key={r.name}>
          <column gap="1">
            <row gap="3" align="baseline">
              <code>{r.name}</code>
              <span style({ box: grow(1), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) })>{r.type}</span>
            </row>
            <if {r.desc !== ''}>
              <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>{r.desc}</p>
            </if>
          </column>
        </for>
      </column>
    </card>
  </if>
</column>
