<!--
  <avatar> — L1 language element. Style value language only.
  Child img pack lives in platform/avatar.css: cover for photo src;
  contain+pad when icon-slug. Default = avatar surface tokens (muted disc).
  Tone is opt-in brand fill.
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

  style base = {position(relative).overflow({ all: hidden }).size(7).fg(foreground).bg(muted).radius(full).elevation({ shadow: inset-ring }).select(none).align(middle).type({ family: body, weight: bold, leading: none, tracking: wide, size: base })}
  style sizeMd = {size(7).type({ size: base })}
  style sizeXs = {size(5).type({ size: xs })}
  style sizeSm = {size(control-h-sm).type({ size: xs })}
  style sizeLg = {size(control-h-lg).type({ size: lg })}
  style sizeXl = {size(control-h-xl).type({ size: 2xl })}
</scope>

<row
  inline
  nowrap
  align="center"
  justify="center"
  use:mediaFallback
  style(base, {shrink(0), match: {size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg, xl: sizeXl }, shape: {round: radius(full), square: radius(sm)}}, compound: [
      {tone: *, bg(tone).fg(tone.foreground)},
      {tone: muted, bg(muted).fg(foreground)},
    ]})
></row>
