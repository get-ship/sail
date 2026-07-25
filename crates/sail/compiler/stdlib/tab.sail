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

  const base = style.fontFamily(body).fontWeight(medium).leading(none).fg(tab-foreground).bg(transparent).cursor(pointer).whitespace(nowrap).select(none).fontSize(sm).padY(2).padX(3).borderWidth(0).borderBWidth(2).borderB(transparent).radiusT(sm).tapHighlight(none).transition(colors).duration(fast).ease(out)
  const sizeMd = style.fontSize(sm).padY(2).padX(3)
  const sizeSm = style.fontSize(xs).padY(1_5).padX(2_5)
  const sizeLg = style.fontSize(base).padY(2_5).padX(4)
  const pill = style.borderBWidth(0).radius(sm)
  const enclosed = style.borderWidth(1).border(transparent).borderBWidth(0).radiusT(md)
</scope>

<button type="button" sx={[
  base,
  match(size, {
    md: sizeMd,
    sm: sizeSm,
    lg: sizeLg,
  }),
  match(variant, {
    line: style.borderB(transparent),
    pill: pill,
    enclosed: enclosed,
  }),
  compound({ variant: pill }, style.borderBWidth(0).radius(sm)),
  compound({ variant: enclosed }, style.borderWidth(1).border(transparent).borderBWidth(0).radiusT(md)),
  when({
    hover: style.fg(foreground),
    selected: style.fg(tab-selected-foreground).fontWeight(semibold).borderB(tone),
    focus: style.outline(solid).outline(ring-width).outline(tone-ring).outlineOffset(ring-inset),
    disabled: style.opacity(50).cursor(not-allowed),
  }),
  compound({ variant: pill }, when({
    selected: style.fg(tab-selected-foreground).bg(tab-selected).shadow(xs).borderB(transparent),
  })),
  compound({ variant: enclosed }, when({
    selected: style.fg(tab-selected-foreground).border(border).bg(tab-selected).mb(-px),
  })),
]}>
  <slot></slot>
</button>
