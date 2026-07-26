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

  const base = style.position(relative).fontFamily(body).fontWeight(bold).leading(none).tracking(wide).fg(avatar-foreground).overflow(hidden).select(none).align(middle).size(7).fontSize(base).radius(full).shadow(inset-ring).bgImage(primary-accent).avatarMedia(true)
  const sizeMd = style.size(7).fontSize(base)
  const sizeXs = style.size(5).fontSize(xs)
  const sizeSm = style.size(control-h-sm).fontSize(xs)
  const sizeLg = style.size(control-h-lg).fontSize(lg)
  const sizeXl = style.size(control-h-xl).fontSize(2xl)
</scope>

<row inline nowrap align="center" justify="center" shrink="0" use:mediaFallback sx={[
  base,
  match(size, {
    md: sizeMd,
    xs: sizeXs,
    sm: sizeSm,
    lg: sizeLg,
    xl: sizeXl,
  }),
  match(shape, {
    round: style.radius(full),
    square: style.radius(sm),
  }),
  match(tone, {
    default: style.bgImage(primary-accent).fg(avatar-foreground),
    primary: style.bgImage(none).bg(primary).fg(primary-foreground),
    accent: style.bgImage(none).bg(accent).fg(accent-foreground),
    info: style.bgImage(none).bg(info).fg(info-foreground),
    success: style.bgImage(none).bg(success).fg(success-foreground),
    warning: style.bgImage(none).bg(warning).fg(warning-foreground),
    destructive: style.bgImage(none).bg(destructive).fg(destructive-foreground),
    muted: style.bgImage(none).bg(avatar).fg(avatar-foreground),
  }),
]}></row>
