<!--
  <toggle> — first/last join + variant/orientation/size Style.
  Paint: rest bg/fg/border from component.toggle tokens
         (cssNames toggle, toggle-foreground, toggle-border).
-->

<scope>
  props: {
    value = ''
    disabled = false
    variant: 'outline' | 'ghost' | 'segmented' = 'outline'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  const base = style.position(relative).fontFamily(body).fontWeight(medium).leading(none).fg(toggle-foreground).bg(toggle).cursor(pointer).whitespace(nowrap).select(none).fontSize(sm).h(control-h-md).padX(3).borderWidth(1).border(toggle-border).radius(none).ms(-px).tapHighlight(none).transition(colors).duration(fast).ease(out)
  const sizeMd = style.fontSize(sm).h(control-h-md).padX(3)
  const sizeSm = style.fontSize(xs).h(control-h-sm).padX(2_5)
  const sizeLg = style.fontSize(base).h(control-h-lg).padX(4)
  const outline = style.radius(none).ms(-px)
  const ghost = style.border(transparent).radius(md).ms(0).me(1)
  const segmented = style.border(transparent).radius(sm).ms(0).me(1)
</scope>

<button type="button" sx={[
  base,
  match(size, {
    md: sizeMd,
    sm: sizeSm,
    lg: sizeLg,
  }),
  match(variant, {
    outline: outline,
    ghost: ghost,
    segmented: segmented,
  }),
  compound({ orientation: vertical, variant: outline }, style.radius(none).mt(-px).ms(0).me(0)),
  compound({ variant: ghost }, style.border(transparent).radius(md).ms(0).mt(0).me(1)),
  compound({ variant: segmented }, style.border(transparent).radius(sm).ms(0).mt(0).me(1)),
  compound({ orientation: vertical, variant: ghost }, style.border(transparent).radius(md).ms(0).mt(0).me(0).mb(1)),
  compound({ orientation: vertical, variant: segmented }, style.border(transparent).radius(sm).ms(0).mt(0).me(0).mb(1)),
  when({
    hover: style.bg(muted),
    pressed: style.bg(foreground).fg(background).border(foreground).z(1),
    disabled: style.opacity(50).cursor(not-allowed),
    focus: style.outline(solid).outline(2).outline(ring).z(2),
  }),
  compound({ orientation: horizontal, variant: outline }, when({
    first: style.ms(0).radiusL(md),
    last: style.radiusR(md),
  })),
  compound({ orientation: vertical, variant: outline }, when({
    first: style.mt(0).radiusT(md),
    last: style.radiusB(md),
  })),
  compound({ variant: ghost }, when({
    last: style.me(0),
  })),
  compound({ variant: segmented }, when({
    last: style.me(0),
  })),
  compound({ orientation: vertical, variant: ghost }, when({
    last: style.mb(0),
  })),
  compound({ orientation: vertical, variant: segmented }, when({
    last: style.mb(0),
  })),
]}>
  <slot></slot>
</button>
