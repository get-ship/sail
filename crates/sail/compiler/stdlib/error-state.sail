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

  const base = style({ box: padY(8).padX(4), paint: fg(error-state-foreground).borderWidth(1).border(error-state-border).radius(lg).bg(error-state).shadow(xs).group(), text: fontFamily(body).textAlign(center) })
  const sizeMd = style({ box: padY(8).padX(4) })
  const sizeSm = style({ box: padY(7) })
  const sizeLg = style({ box: padY(8).padX(6) })
  const variantDefault = style({ paint: borderWidth(1).border(error-state-border).radius(lg).bg(error-state).shadow(xs), text: textAlign(center) })
  const variantInline = style({ box: padY(3).padX(4), paint: borderWidth(1).border(destructive).radius(md).bg(muted).shadow(none), text: textAlign(start) })
  const icon = style({ box: self(center).size(control-h-xl), paint: radius(lg).bg(muted).fg(destructive).outline(1).outline(solid).outline(destructive).outlineOffset(ring-inset) })
  const glyph = style({ box: size(control-h-2xs) })
  const copy = style({ box: self(stretch) })
  const title = style({ paint: fg(foreground), text: fontFamily(heading).fontWeight(semibold).leading(tight).fontSize(md) })
  const desc = style({ box: maxW(panel-ch-w-md), paint: fg(muted-foreground), text: leading(loose).fontSize(sm) })
  const codeBox = style({ box: padY(1).padX(2).mt(1), paint: fg(muted-foreground).bg(muted).borderWidth(1).border(border-subtle).radius(sm), text: fontFamily(mono).fontSize(xs) })
  const detailsToggle = style({ box: display(none).mt(1).padY(1).padX(2), paint: borderWidth(0).bg(transparent).fg(muted-foreground).radius(sm).cursor(pointer), text: fontFamily(body).fontSize(xs) })
  const details = style({ box: display(none).mt(2).padY(2).padX(3).maxW(panel-ch-w-lg), paint: bg(muted).radius(sm).fg(muted-foreground), text: fontFamily(mono).fontSize(xs).whitespace(pre-wrap).break(words).textAlign(start) })
  const actions = style({ box: self(center) })
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:recoverySurface style([
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
      forced-colors: style({ paint: border(CanvasText) }),
    }),
  ])>
  <row inline nowrap align="center" justify="center" hidden="true" style([ style({ box: shrink(1) }), 
      icon,
      when({
        forced-colors: style({ paint: bg(Canvas).fg(CanvasText) }),
      }),
    ])>
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" style(glyph)>
      <circle cx="12" cy="12" r="9"></circle>
      <path d="M12 8v5"></path>
      <path d="M12 16h.01"></path>
    </svg>
  </row>
  <column align="stretch" gap="1" style([ style(copy), style({ box: grow(1).minW(0) }) ])>
    <div style(title)>
      <slot name="title">Something went wrong</slot>
    </div>
    <div style([
      desc,
      when({
        empty: style({ box: display(none) }),
      }),
    ])>
      <slot></slot>
    </div>
    <div style([
      codeBox,
      when({
        empty: style({ box: display(none) }),
      }),
    ])></div>
    <button
      type="button"
      anatomy
      expanded="false" style([
        detailsToggle,
        when({
          group-state-has-details: style({ box: display(inline-block) }),
          hover: style({ paint: fg(foreground).bg(muted) }),
          focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(2) }),
        }),
      ])>Show details</button>
    <div style([
      details,
      when({
        group-state-open: style({ box: display(block) }),
      }),
    ])>
      <slot name="details"></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" style([
    actions,
    when({
      empty: style({ box: display(none) }),
    }),
  ])>
    <slot name="action"></slot>
  </row>
</column>
