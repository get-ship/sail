<!--
  <toggle> — pressed-button cell for <toggle-group> only (standalone died).
  First/last join + variant/orientation/size Style.
  Binary form control → use <switch>. Action button → use <button>.
  Selected = primary tokens (not raw fg/bg invert); pressed+hover keeps identity.
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
  style base = {
    box: position(relative).display(inline-flex).h(control-h-md).padX(3).ms(-px),
    paint: appearance(none).fg(toggle.foreground).bg(toggle).cursor(pointer).select(none).borderWidth(1).border(solid).border(toggle.border).radius(none).tapHighlight(none).transition(colors).duration(fast).ease(out),
    text: fontFamily(body).fontWeight(medium).leading(none).whitespace(nowrap).fontSize(sm),
  }
  style sizeMd = { box: h(control-h-md).padX(3), text: fontSize(sm) }
  style sizeSm = { box: h(control-h-sm).padX(2_5), text: fontSize(xs) }
  style sizeLg = { box: h(control-h-lg).padX(4), text: fontSize(base) }
  style outline = { box: ms(-px), paint: radius(none) }
  style ghost = { box: ms(0).me(1), paint: border(transparent).radius(md) }
  style segmented = { box: ms(0).me(1), paint: border(transparent).radius(sm) }
</scope>

<button
  type="button"
  style(base, {
    when: {
      hover: { paint: bg(hover-bg).fg(foreground) },
      pressed: {
        paint: bg(primary).fg(primary-foreground).border(primary).z(1),
        when: {
          // Selected must survive hover (not hover-bg wash / unselected look).
          hover: { paint: bg(primary).fg(primary-foreground).border(primary).opacity(90) },
        },
      },
      disabled: { paint: opacity(50).cursor(not-allowed) },
      focus: { paint: outline(solid).outline(2).outline(ring).z(2) },
    },
    match: {
      size: { md: sizeMd, sm: sizeSm, lg: sizeLg },
      variant: { outline: outline, ghost: ghost, segmented: segmented },
    },
    compound: [
      { orientation: vertical, variant: outline, box: mt(-px).ms(0).me(0), paint: radius(none) },
      { variant: ghost, box: ms(0).mt(0).me(1), paint: border(transparent).radius(md) },
      { variant: segmented, box: ms(0).mt(0).me(1), paint: border(transparent).radius(sm) },
      { orientation: vertical, variant: ghost, box: ms(0).mt(0).me(0).mb(1), paint: border(transparent).radius(md) },
      { orientation: vertical, variant: segmented, box: ms(0).mt(0).me(0).mb(1), paint: border(transparent).radius(sm) },
      { orientation: horizontal, variant: outline, when: { first: { box: ms(0), paint: radiusL(md) }, last: { paint: radiusR(md) } } },
      { orientation: vertical, variant: outline, when: { first: { box: mt(0), paint: radiusT(md) }, last: { paint: radiusB(md) } } },
      { variant: ghost, when: { last: { box: me(0) } } },
      { variant: segmented, when: { last: { box: me(0) } } },
      { orientation: vertical, variant: ghost, when: { last: { box: mb(0) } } },
      { orientation: vertical, variant: segmented, when: { last: { box: mb(0) } } },
    ],
  })
>
  <slot></slot>
</button>
