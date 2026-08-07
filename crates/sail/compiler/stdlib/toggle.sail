<!--
  <toggle> — pressed-button cell for <toggle-group> only (standalone died).
  First/last join + variant/orientation/size Style.
  Paint: rest bg/fg/border from component.toggle tokens
         (cssNames toggle, toggle-foreground, toggle-border).
  Binary form control → use <switch>. Action button → use <button>.
-->

<scope>
  props: {
    value = ''
    disabled = false
    variant: 'outline' | 'ghost' | 'segmented' = 'outline'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  // border(solid) is required — bare <button> UA is border-style:outset; without
  // it the outline join paints as separate browser chrome (not a segment).
  const base = style({ box: position(relative).display(inline-flex).h(control-h-md).padX(3).ms(-px), paint: appearance(none).fg(toggle-foreground).bg(toggle).cursor(pointer).select(none).borderWidth(1).border(solid).border(toggle-border).radius(none).tapHighlight(none).transition(colors).duration(fast).ease(out), text: fontFamily(body).fontWeight(medium).leading(none).whitespace(nowrap).fontSize(sm) })
  const sizeMd = style({ box: h(control-h-md).padX(3), text: fontSize(sm) })
  const sizeSm = style({ box: h(control-h-sm).padX(2_5), text: fontSize(xs) })
  const sizeLg = style({ box: h(control-h-lg).padX(4), text: fontSize(base) })
  const outline = style({ box: ms(-px), paint: radius(none) })
  const ghost = style({ box: ms(0).me(1), paint: border(transparent).radius(md) })
  const segmented = style({ box: ms(0).me(1), paint: border(transparent).radius(sm) })
</scope>

<button type="button" style([
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
  compound({ orientation: vertical, variant: outline }, style({ box: mt(-px).ms(0).me(0), paint: radius(none) })),
  compound({ variant: ghost }, style({ box: ms(0).mt(0).me(1), paint: border(transparent).radius(md) })),
  compound({ variant: segmented }, style({ box: ms(0).mt(0).me(1), paint: border(transparent).radius(sm) })),
  compound({ orientation: vertical, variant: ghost }, style({ box: ms(0).mt(0).me(0).mb(1), paint: border(transparent).radius(md) })),
  compound({ orientation: vertical, variant: segmented }, style({ box: ms(0).mt(0).me(0).mb(1), paint: border(transparent).radius(sm) })),
  // hover-bg (not muted) — muted collapses into the surface on many dark
  // themes and makes unpressed hover text unreadable.
  when({
    hover: style({ paint: bg(hover-bg).fg(foreground) }),
    pressed: style({ paint: bg(foreground).fg(background).border(foreground).z(1) }),
    disabled: style({ paint: opacity(50).cursor(not-allowed) }),
    focus: style({ paint: outline(solid).outline(2).outline(ring).z(2) }),
  }),
  compound({ orientation: horizontal, variant: outline }, when({
    first: style({ box: ms(0), paint: radiusL(md) }),
    last: style({ paint: radiusR(md) }),
  })),
  compound({ orientation: vertical, variant: outline }, when({
    first: style({ box: mt(0), paint: radiusT(md) }),
    last: style({ paint: radiusB(md) }),
  })),
  compound({ variant: ghost }, when({
    last: style({ box: me(0) }),
  })),
  compound({ variant: segmented }, when({
    last: style({ box: me(0) }),
  })),
  compound({ orientation: vertical, variant: ghost }, when({
    last: style({ box: mb(0) }),
  })),
  compound({ orientation: vertical, variant: segmented }, when({
    last: style({ box: mb(0) }),
  })),
])>
  <slot></slot>
</button>
