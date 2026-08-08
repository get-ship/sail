<!--
  <avatar> — L1 language element. Style value language only.
  avatarMedia child pack sizes injected img.
  Rest paint = primary-accent gradient when tone is omitted.
  Semantic paint = bound tone group paths. Authors omit tone for rest (never tone="default").
-->

<scope>
  props: {
    src = ''
    'icon-slug' = ''
    name = ''
    initials = ''
    tone: 'primary' | 'accent' | 'info' | 'success' | 'warning' | 'destructive' | 'muted'
    size: 'md' | 'xs' | 'sm' | 'lg' | 'xl' = 'md'
    shape: 'round' | 'square' = 'round'
  }

  style base = { box: position(relative).overflow(hidden).size(7), paint: fg(avatar.foreground).select(none).align(middle).radius(full).shadow(inset-ring).bgImage(primary-accent).avatarMedia(true), text: fontFamily(body).fontWeight(bold).leading(none).tracking(wide).fontSize(base) }
  style sizeMd = { box: size(7), text: fontSize(base) }
  style sizeXs = { box: size(5), text: fontSize(xs) }
  style sizeSm = { box: size(control-h-sm), text: fontSize(xs) }
  style sizeLg = { box: size(control-h-lg), text: fontSize(lg) }
  style sizeXl = { box: size(control-h-xl), text: fontSize(2xl) }
</scope>

<row inline nowrap align="center" justify="center" use:mediaFallback style(base, { box: shrink(0), paint: bgImage(none).bg(tone).fg(tone.foreground), match: { size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg, xl: sizeXl }, shape: { round: { paint: radius(full) }, square: { paint: radius(sm) } } }, compound: [ { tone: muted, paint: bgImage(none).bg(avatar).fg(avatar.foreground) } ] })></row>
