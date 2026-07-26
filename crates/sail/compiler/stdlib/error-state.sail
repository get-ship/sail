<!--
  <error-state> — L1 recovery surface. Style value language only.
  Destructive paint via semantic tokens (no tone-* channels).
-->

<scope>
  props: {
    variant: 'default' | 'inline' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
    code = ''
  }

  const base = style.fontFamily(body).fg(error-state-foreground).textAlign(center).borderWidth(1).border(error-state-border).radius(lg).bg(error-state).shadow(xs).padY(8).padX(4).group()
  const sizeMd = style.padY(8).padX(4)
  const sizeSm = style.padY(7)
  const sizeLg = style.padY(8).padX(6)
  const variantDefault = style.textAlign(center).borderWidth(1).border(error-state-border).radius(lg).bg(error-state).shadow(xs)
  const variantInline = style.textAlign(start).borderWidth(1).border(destructive).radius(md).bg(muted).shadow(none).padY(3).padX(4)
  const icon = style.self(center).radius(lg).bg(muted).fg(destructive).size(control-h-xl).outline(1).outline(solid).outline(destructive).outlineOffset(ring-inset)
  const glyph = style.size(control-h-2xs)
  const copy = style.self(stretch)
  const title = style.fontFamily(heading).fontWeight(semibold).fg(foreground).leading(tight).fontSize(md)
  const desc = style.leading(loose).fg(muted-foreground).fontSize(sm).maxW(panel-ch-w-md)
  const codeBox = style.fontFamily(mono).fontSize(xs).fg(muted-foreground).bg(muted).borderWidth(1).border(border-subtle).radius(sm).padY(1).padX(2).mt(1)
  const detailsToggle = style.display(none).mt(1).padY(1).padX(2).borderWidth(0).bg(transparent).fg(muted-foreground).fontFamily(body).fontSize(xs).radius(sm).cursor(pointer)
  const details = style.display(none).mt(2).padY(2).padX(3).bg(muted).radius(sm).fontFamily(mono).fontSize(xs).fg(muted-foreground).whitespace(pre-wrap).break(words).textAlign(start).maxW(panel-ch-w-lg)
  const actions = style.self(center)
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:recoverySurface
  role="alert"
  sx={[
    base,
    match(size, {
      md: sizeMd,
      sm: sizeSm,
      lg: sizeLg,
    }),
    match(variant, {
      default: variantDefault,
      inline: variantInline,
    }),
    when({
      forced-colors: style.border(CanvasText),
    }),
  ]}
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    class="icon"
    aria-hidden="true"
    shrink
    sx={[
      icon,
      when({
        forced-colors: style.bg(Canvas).fg(CanvasText),
      }),
    ]}
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" sx={glyph}>
      <circle cx="12" cy="12" r="9"></circle>
      <path d="M12 8v5"></path>
      <path d="M12 16h.01"></path>
    </svg>
  </row>
  <column align="stretch" gap="1" class="copy" grow min-w="0" sx={copy}>
    <div class="title" sx={title}>
      <slot name="title">Something went wrong</slot>
    </div>
    <div class="desc" sx={[
      desc,
      when({
        empty: style.display(none),
      }),
    ]}>
      <slot></slot>
    </div>
    <div class="code" sx={[
      codeBox,
      when({
        empty: style.display(none),
      }),
    ]}></div>
    <button
      class="details-toggle"
      type="button"
      anatomy
      aria-expanded="false"
      sx={[
        detailsToggle,
        when({
          group-state-has-details: style.display(inline-block),
          hover: style.fg(foreground).bg(muted),
          focus: style.outline(solid).outline(2).outline(ring).outlineOffset(2),
        }),
      ]}
    >Show details</button>
    <div class="details" sx={[
      details,
      when({
        group-state-open: style.display(block),
      }),
    ]}>
      <slot name="details"></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" class="actions" sx={[
    actions,
    when({
      empty: style.display(none),
    }),
  ]}>
    <slot name="action"></slot>
  </row>
</column>
