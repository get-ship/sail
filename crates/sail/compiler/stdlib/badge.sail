<!--
  <badge> — L1 language element (element mode). A small, static, tone-colored
  status label. No behavior, no control flow — pure surface.

  Paint (one path):
    host sx={…}    — soft base + size md + match(variant) + compound(pill)
    match          — variant prop branch (replaces CVA-in-props)
    compound       — pill → rounded-full; soft+default → muted soft
    when           — hover/focus states via CSS pseudo
    SSOT tokens    — all paint values are token names from the registry
  Zero CVA-in-props. Zero tone-* happy-path paint. Zero legacy string sx.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    variant: 'soft' | 'solid' | 'outline' = 'soft'
    size: 'sm' | 'md' | 'lg' = 'md'
    pill = false
  }
</scope>

<span sx={[
  style.fontFamily(body).fontWeight(semibold).leading(none).whitespace(nowrap).align(middle).tabularNums().antialiased().border().radius(sm).bg(toneBg).color(toneFg).borderColor(toneBorder).minH(controlH2xs).padX(2).fontSize(xs),
  match(variant, {
    soft: style.bg(toneBg).color(toneFg).borderColor(toneBorder),
    solid: style.bg(tone).color(toneContrast).borderColor(tone).shadow(xs),
    outline: style.bg(transparent).color(toneFg).borderColor(toneBorder),
  }),
  match(size, {
    md: style.minH(controlH2xs).padX(2).fontSize(xs),
    sm: style.minH(4).padX(1_5).fontSize(xs),
    lg: style.minH(controlHXs).padX(2_5).fontSize(xs),
  }),
  compound({ pill: true }, style.radius(full)),
  compound({ variant: 'soft', tone: 'default' }, style.bg(toneMutedBg).color(mutedForeground).borderColor(toneMutedBorder)),
  when({
    hover: style.bg(toneBgHover),
    focus: style.ring(focusRing),
  }),
]}>
  <slot></slot>
</span>
