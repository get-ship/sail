<!--
  <badge> — L1 language element. Style value language only:
  style.* chain + match + when + compound + scope Style consts.
  SSOT tokens (no tone-*, no CVA maps).
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    variant: 'soft' | 'solid' | 'outline' = 'soft'
    size: 'md' | 'sm' | 'lg' = 'md'
    pill = false
  }

  const base = style({ box: display(inline-flex).minH(control-h-2xs).padX(2).padY(px), paint: borderWidth(1).radius(sm), text: fontFamily(body).fontWeight(semibold).leading(none).whitespace(nowrap).fontSize(xs) })

  // Soft + tone=default ride component.badge tokens (pack defaults alias muted).
  const soft = style({ paint: bg(badge).fg(badge-foreground).border(badge-border) })
  const solid = style({ paint: bg(primary).fg(primary-foreground).border(primary).shadow(xs) })
  const outline = style({ paint: bg(transparent).fg(badge-foreground).border(badge-border) })

  const sizeMd = style({ box: minH(control-h-2xs).padX(2), text: fontSize(xs) })
  const sizeSm = style({ box: minH(4).padX(1_5), text: fontSize(xs) })
  const sizeLg = style({ box: minH(control-h-xs).padX(2_5), text: fontSize(xs) })

  const toneDefault = style({ paint: bg(badge).fg(badge-foreground).border(badge-border) })
  const tonePrimary = style({ paint: bg(primary).fg(primary-foreground).border(primary) })
  const toneInfo = style({ paint: bg(info).fg(info-foreground).border(info) })
  const toneSuccess = style({ paint: bg(success).fg(success-foreground).border(success) })
  const toneWarning = style({ paint: bg(warning).fg(warning-foreground).border(warning) })
  const toneDestructive = style({ paint: bg(destructive).fg(destructive-foreground).border(destructive) })
  const toneAccent = style({ paint: bg(accent).fg(accent-foreground).border(accent) })
  const toneMuted = style({ paint: bg(muted).fg(muted-foreground).border(border) })
</scope>

<span>
  style([
  base,
  match(variant, {
    soft: soft,
    solid: solid,
    outline: outline,
  }),
  match(size, {
    md: sizeMd,
    sm: sizeSm,
    lg: sizeLg,
  }),
  match(tone, {
    default: toneDefault,
    primary: tonePrimary,
    info: toneInfo,
    success: toneSuccess,
    warning: toneWarning,
    destructive: toneDestructive,
    accent: toneAccent,
    muted: toneMuted,
  }),
  compound({ pill: true }, style({ paint: radius(full) })),
  compound({ variant: soft, tone: default }, style({ paint: bg(badge).fg(badge-foreground).border(badge-border) })),
  when({
    hover: style({ paint: opacity(90) }),
    focus: style({ paint: outline(2) }),
  }),
])

  <slot></slot>
</span>
