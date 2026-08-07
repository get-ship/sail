<!--
  <badge> — L1 language element. Style value language only.
  Rest paint = component.badge tokens when tone is omitted.
  Semantic paint = bound tone group paths. Authors omit tone for rest (never tone="default").
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    variant: 'soft' | 'solid' | 'outline' = 'soft'
    size: 'md' | 'sm' | 'lg' = 'md'
    pill = false
  }

  const base = style({ box: display(inline-flex).minH(control-h-2xs).padX(2).padY(px), paint: borderWidth(1).radius(sm), text: fontFamily(body).fontWeight(semibold).leading(none).whitespace(nowrap).fontSize(xs) })

  // Rest variants ride component.badge tokens (pack defaults).
  const soft = style({ paint: bg(badge).fg(badge-foreground).border(badge-border) })
  const solid = style({ paint: bg(badge).fg(badge-foreground).border(badge-border).shadow(xs) })
  const outline = style({ paint: bg(transparent).fg(badge-foreground).border(badge-border) })

  const sizeMd = style({ box: minH(control-h-2xs).padX(2), text: fontSize(xs) })
  const sizeSm = style({ box: minH(4).padX(1_5), text: fontSize(xs) })
  const sizeLg = style({ box: minH(control-h-xs).padX(2_5), text: fontSize(xs) })

  // Brand when tone= is set — expands over Tone groups with [tone='…'] gates.
  const solidTone = style({ paint: bg(tone).fg(tone.foreground).border(tone).shadow(xs) })
  const softTone = style({ paint: colorMix(tone, 15).border(tone).fg(tone) })
  const outlineTone = style({ paint: bg(transparent).fg(tone).border(tone) })
</scope>

<span style([
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
  match(variant, {
    solid: solidTone,
    soft: softTone,
    outline: outlineTone,
  }),
  // Muted specials — not pure group-member paint for soft/outline.
  compound({ variant: solid, tone: muted }, style({ paint: bg(muted).fg(muted-foreground).border(border) })),
  compound({ variant: soft, tone: muted }, style({ paint: bg(muted).fg(muted-foreground).border(border) })),
  compound({ variant: outline, tone: muted }, style({ paint: fg(muted-foreground).border(border) })),
  compound({ pill: true }, style({ paint: radius(full) })),
  when({
    hover: style({ paint: opacity(90) }),
    focus: style({ paint: outline(2) }),
  }),
])>
  <slot></slot>
</span>
