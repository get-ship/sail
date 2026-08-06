<!--
  <button> — native <button>. L1 platform control. Style value language only.

  Paint + size live HERE. Flex geometry (inline-flex / align-items / gap) and
  Tier-2 chrome (control-frame / corner-image / lift / press) that Style already
  owns live here too. Focus ring + disabled opacity stay on the shared document
  baseline — not a parallel BUTTON_SKIN_CSS residual.
-->

<scope>
  props: {
    variant: 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'default'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    disabled = false
  }

  // border(solid) kills UA button outset; without it freeform demos look like
  // native OS chrome instead of design-system controls.
  //
  // No base bg(transparent): that class is a cascade landmine (F7). Unlayered
  // product `.sx-bgtransparent` beats layered solid skins and empties primary
  // fills. Variants own background; ghost is the only transparent fill.
  const base = style({ box: display(inline-flex).position(relative), paint: appearance(none).radius(md).borderWidth(control-stroke).border(solid).border(transparent).borderImage(corner-image).isolate().controlFrame().select(none).cursor(pointer), text: fontWeight(medium).leading(none).whitespace(nowrap) }).when(hover, { paint: controlLift() }).when(active, { paint: press() }).when(disabled, { paint: cursor(not-allowed).opacity(50) })

  const sizeXs = style({ box: minH(control-h-2xs).padX(2), text: fontSize(xs) })
  const sizeSm = style({ box: minH(control-h-sm).padX(2_5), text: fontSize(sm) })
  const sizeMd = style({ box: minH(control-h-md).padX(3), text: fontSize(base) })
  const sizeLg = style({ box: minH(control-h-lg).padX(4), text: fontSize(md) })

  const vDefault = style({ paint: bg(background).border(border).fg(foreground).shadow(xs) }).when(hover, { paint: bg(muted).border(border-strong) })
  const vSolid = style({ paint: bg(foreground).border(foreground).fg(background).shadow(xs) }).when(hover, { paint: opacity(90).shadow(sm) })
  const vSoft = style({ paint: bg(hover-bg).border(border).fg(foreground) }).when(hover, { paint: opacity(90) })
  const vOutline = style({ paint: border(foreground).fg(foreground) }).when(hover, { paint: bg(hover-bg) }).when(disabled, { paint: bg(transparent) })
  const vGhost = style({ paint: bg(transparent).fg(foreground) }).when(hover, { paint: bg(hover-bg) }).when(disabled, { paint: bg(transparent) })
</scope>

<button>
  style([
  base,
  match(variant, {
    default: vDefault,
    solid: vSolid,
    soft: vSoft,
    outline: vOutline,
    ghost: vGhost,
  }),
  match(size, {
    xs: sizeXs,
    sm: sizeSm,
    md: sizeMd,
    lg: sizeLg,
  }),
  compound({ variant: solid, tone: primary }, style({ paint: bg(primary).border(primary).fg(primary-foreground) })),
  compound({ variant: solid, tone: info }, style({ paint: bg(info).border(info).fg(info-foreground) })),
  compound({ variant: solid, tone: success }, style({ paint: bg(success).border(success).fg(success-foreground) })),
  compound({ variant: solid, tone: warning }, style({ paint: bg(warning).border(warning).fg(warning-foreground) })),
  compound({ variant: solid, tone: destructive }, style({ paint: bg(destructive).border(destructive).fg(destructive-foreground) })),
  compound({ variant: solid, tone: accent }, style({ paint: bg(accent).border(accent).fg(accent-foreground) })),
  compound({ variant: solid, tone: muted }, style({ paint: bg(muted-foreground).border(muted-foreground).fg(background) })),
  compound({ variant: soft, tone: primary }, style({ paint: colorMix(primary, 15).border(primary).fg(primary) }).when(hover, { paint: colorMix(primary, 25) })),
  compound({ variant: soft, tone: info }, style({ paint: colorMix(info, 15).border(info).fg(info) }).when(hover, { paint: colorMix(info, 25) })),
  compound({ variant: soft, tone: success }, style({ paint: colorMix(success, 15).border(success).fg(success) }).when(hover, { paint: colorMix(success, 25) })),
  compound({ variant: soft, tone: warning }, style({ paint: colorMix(warning, 15).border(warning).fg(warning) }).when(hover, { paint: colorMix(warning, 25) })),
  compound({ variant: soft, tone: destructive }, style({ paint: colorMix(destructive, 15).border(destructive).fg(destructive) }).when(hover, { paint: colorMix(destructive, 25) })),
  compound({ variant: soft, tone: accent }, style({ paint: colorMix(accent, 15).border(accent).fg(accent) }).when(hover, { paint: colorMix(accent, 25) })),
  compound({ variant: soft, tone: muted }, style({ paint: bg(muted).border(border).fg(muted-foreground) })),
  compound({ variant: outline, tone: primary }, style({ paint: border(primary).fg(primary) })),
  compound({ variant: outline, tone: info }, style({ paint: border(info).fg(info) })),
  compound({ variant: outline, tone: success }, style({ paint: border(success).fg(success) })),
  compound({ variant: outline, tone: warning }, style({ paint: border(warning).fg(warning) })),
  compound({ variant: outline, tone: destructive }, style({ paint: border(destructive).fg(destructive) })),
  compound({ variant: outline, tone: accent }, style({ paint: border(accent).fg(accent) })),
  compound({ variant: outline, tone: muted }, style({ paint: border(muted-foreground).fg(muted-foreground) })),
  compound({ variant: ghost, tone: primary }, style({ paint: fg(primary) })),
  compound({ variant: ghost, tone: info }, style({ paint: fg(info) })),
  compound({ variant: ghost, tone: success }, style({ paint: fg(success) })),
  compound({ variant: ghost, tone: warning }, style({ paint: fg(warning) })),
  compound({ variant: ghost, tone: destructive }, style({ paint: fg(destructive) })),
  compound({ variant: ghost, tone: accent }, style({ paint: fg(accent) })),
  compound({ variant: ghost, tone: muted }, style({ paint: fg(muted-foreground) })),
])

  <slot></slot>
</button>
