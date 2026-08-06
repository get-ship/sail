<!--
  <avatar> — L1 language element. Style value language only.
  avatarMedia child pack sizes injected img.
-->

<scope>
  props: {
    src = ''
    'icon-slug' = ''
    name = ''
    initials = ''
    tone: 'default' | 'primary' | 'accent' | 'info' | 'success' | 'warning' | 'destructive' | 'muted' = 'default'
    size: 'md' | 'xs' | 'sm' | 'lg' | 'xl' = 'md'
    shape: 'round' | 'square' = 'round'
  }

  const base = style({ box: position(relative).overflow(hidden).size(7), paint: fg(avatar-foreground).select(none).align(middle).radius(full).shadow(inset-ring).bgImage(primary-accent).avatarMedia(true), text: fontFamily(body).fontWeight(bold).leading(none).tracking(wide).fontSize(base) })
  const sizeMd = style({ box: size(7), text: fontSize(base) })
  const sizeXs = style({ box: size(5), text: fontSize(xs) })
  const sizeSm = style({ box: size(control-h-sm), text: fontSize(xs) })
  const sizeLg = style({ box: size(control-h-lg), text: fontSize(lg) })
  const sizeXl = style({ box: size(control-h-xl), text: fontSize(2xl) })
</scope>

<row inline nowrap align="center" justify="center" shrink="0" use:mediaFallback>
  style([
  base,
  match(size, {
    md: sizeMd,
    xs: sizeXs,
    sm: sizeSm,
    lg: sizeLg,
    xl: sizeXl,
  }),
  match(shape, {
    round: style({ paint: radius(full) }),
    square: style({ paint: radius(sm) }),
  }),
  match(tone, {
    default: style({ paint: bgImage(primary-accent).fg(avatar-foreground) }),
    primary: style({ paint: bgImage(none).bg(primary).fg(primary-foreground) }),
    accent: style({ paint: bgImage(none).bg(accent).fg(accent-foreground) }),
    info: style({ paint: bgImage(none).bg(info).fg(info-foreground) }),
    success: style({ paint: bgImage(none).bg(success).fg(success-foreground) }),
    warning: style({ paint: bgImage(none).bg(warning).fg(warning-foreground) }),
    destructive: style({ paint: bgImage(none).bg(destructive).fg(destructive-foreground) }),
    muted: style({ paint: bgImage(none).bg(avatar).fg(avatar-foreground) }),
  }),
])
</row>
