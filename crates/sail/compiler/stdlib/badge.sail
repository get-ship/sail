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

  const base = style.display(inline-flex).fontFamily(body).fontWeight(semibold).leading(none).whitespace(nowrap).borderWidth(1).radius(sm).minH(control-h-2xs).padX(2).padY(px).fontSize(xs)

  const soft = style.bg(muted).fg(muted-foreground).border(border)
  const solid = style.bg(primary).fg(primary-foreground).border(primary).shadow(xs)
  const outline = style.bg(transparent).fg(foreground).border(border)

  const sizeMd = style.minH(control-h-2xs).padX(2).fontSize(xs)
  const sizeSm = style.minH(4).padX(1_5).fontSize(xs)
  const sizeLg = style.minH(control-h-xs).padX(2_5).fontSize(xs)

  const toneDefault = style.bg(muted).fg(muted-foreground).border(border)
  const tonePrimary = style.bg(primary).fg(primary-foreground).border(primary)
  const toneInfo = style.bg(info).fg(info-foreground).border(info)
  const toneSuccess = style.bg(success).fg(success-foreground).border(success)
  const toneWarning = style.bg(warning).fg(warning-foreground).border(warning)
  const toneDestructive = style.bg(destructive).fg(destructive-foreground).border(destructive)
  const toneAccent = style.bg(accent).fg(accent-foreground).border(accent)
  const toneMuted = style.bg(muted).fg(muted-foreground).border(border)
</scope>

<span sx={[
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
  compound({ pill: true }, style.radius(full)),
  compound({ variant: soft, tone: default }, style.bg(muted).fg(muted-foreground).border(border)),
  when({
    hover: style.opacity(90),
    focus: style.outline(2),
  }),
]}>
  <slot></slot>
</span>
