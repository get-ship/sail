<!--
  <avatar> — L1 language element. Style value language only.
  avatarMedia child pack: cover for photo src; contain+pad when icon-slug.
  Default = avatar surface tokens (muted disc). Tone is opt-in brand fill.
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

  style base = {box: place({ position: relative }).overflow({ all: hidden }).size(7), paint: ink({ color: avatar.foreground }).fill({ color: avatar }).radius(full).elevation({ shadow: inset-ring }).select(none).align(middle).avatarMedia(true), text: type({ family: body, weight: bold, leading: none, tracking: wide, size: base })}
  style sizeMd = {box: size(7), text: type({ size: base })}
  style sizeXs = {box: size(5), text: type({ size: xs })}
  style sizeSm = {box: size(control-h-sm), text: type({ size: xs })}
  style sizeLg = {box: size(control-h-lg), text: type({ size: lg })}
  style sizeXl = {box: size(control-h-xl), text: type({ size: 2xl })}
</scope>

<row
  inline
  nowrap
  align="center"
  justify="center"
  use:mediaFallback
  style(base, {box: flow({ shrink: 0 }), match: {size: {
        md: sizeMd,
        xs: sizeXs,
        sm: sizeSm,
        lg: sizeLg,
        xl: sizeXl,
      }, shape: {round: {paint: radius(full)}, square: {paint: radius(sm)}}}, compound: [
      {tone: *, paint: fill({ color: tone }).ink({ color: tone.foreground })},
      {tone: muted, paint: fill({ color: avatar }).ink({ color: avatar.foreground })},
    ]})
></row>
