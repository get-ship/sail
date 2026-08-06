<!--
  <tab> — paint + variant/size Style match for pill/enclosed and pad scale.
  Paint: idle/selected from component.tab tokens
         (cssNames tab-foreground, tab-selected-foreground, tab-selected).
-->

<scope>
  props: {
    name = ''
    disabled = false
    variant: 'line' | 'pill' | 'enclosed' = 'line'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  const base = style({ box: padY(2).padX(3), paint: fg(tab-foreground).bg(transparent).cursor(pointer).select(none).borderWidth(0).borderBWidth(2).borderB(transparent).radiusT(sm).tapHighlight(none).transition(colors).duration(fast).ease(out), text: fontFamily(body).fontWeight(medium).leading(none).whitespace(nowrap).fontSize(sm) })
  const sizeMd = style({ box: padY(2).padX(3), text: fontSize(sm) })
  const sizeSm = style({ box: padY(1_5).padX(2_5), text: fontSize(xs) })
  const sizeLg = style({ box: padY(2_5).padX(4), text: fontSize(base) })
  const pill = style({ paint: borderBWidth(0).radius(sm) })
  const enclosed = style({ paint: borderWidth(1).border(transparent).borderBWidth(0).radiusT(md) })
</scope>

<button type="button">
  style([
  base,
  match(size, {
    md: sizeMd,
    sm: sizeSm,
    lg: sizeLg,
  }),
  match(variant, {
    line: style({ paint: borderB(transparent) }),
    pill: pill,
    enclosed: enclosed,
  }),
  compound({ variant: pill }, style({ paint: borderBWidth(0).radius(sm) })),
  compound({ variant: enclosed }, style({ paint: borderWidth(1).border(transparent).borderBWidth(0).radiusT(md) })),
  when({
    hover: style({ paint: fg(foreground) }),
    selected: style({ paint: fg(tab-selected-foreground).borderB(primary), text: fontWeight(semibold) }),
    focus: style({ paint: outline(solid).outline(ring-width).outline(ring).outlineOffset(ring-inset) }),
    disabled: style({ paint: opacity(50).cursor(not-allowed) }),
  }),
  compound({ variant: pill }, when({
    selected: style({ paint: fg(tab-selected-foreground).bg(tab-selected).shadow(xs).borderB(transparent) }),
  })),
  compound({ variant: enclosed }, when({
    selected: style({ paint: fg(tab-selected-foreground).border(border).bg(tab-selected).translateY(px) }),
  })),
])

  <slot></slot>
</button>
