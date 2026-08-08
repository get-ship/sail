<!--
  <badge> — L1 language element. Style value language only.
  Rest paint = component.badge tokens when tone is omitted.
  Semantic paint = bound tone.
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    variant: 'soft' | 'solid' | 'outline' = 'soft'
    size: 'md' | 'sm' | 'lg' = 'md'
    pill = false
  }

  style base = { box: display(inline-flex).minH(control-h-2xs).padX(2).padY(px), paint: borderWidth(1).radius(sm), text: fontFamily(body).fontWeight(semibold).leading(none).whitespace(nowrap).fontSize(xs) }

  style soft = { paint: bg(badge).fg(badge.foreground).border(badge.border) }
  style solid = { paint: bg(badge).fg(badge.foreground).border(badge.border).shadow(xs) }
  style outline = { paint: bg(transparent).fg(badge.foreground).border(badge.border) }

  style sizeMd = { box: minH(control-h-2xs).padX(2), text: fontSize(xs) }
  style sizeSm = { box: minH(4).padX(1_5), text: fontSize(xs) }
  style sizeLg = { box: minH(control-h-xs).padX(2_5), text: fontSize(xs) }

  style solidTone = { paint: bg(tone).fg(tone.foreground).border(tone).shadow(xs) }
  style softTone = { paint: colorMix(tone, 15).border(tone).fg(tone) }
  style outlineTone = { paint: bg(transparent).fg(tone).border(tone) }
</scope>

<span
  style(base, {
    when: {
      hover: { paint: opacity(90) },
      focus: { paint: outline(2) },
    },
    match: {
      variant: {
        soft: style(soft, softTone),
        solid: style(solid, solidTone),
        outline: style(outline, outlineTone),
      },
      size: {
        md: sizeMd,
        sm: sizeSm,
        lg: sizeLg,
      },
    },
    compound: [
      { tone: muted, variant: solid, paint: bg(muted).fg(muted-foreground).border(border) },
      { tone: muted, variant: soft, paint: bg(muted).fg(muted-foreground).border(border) },
      { tone: muted, variant: outline, paint: fg(muted-foreground).border(border) },
      { pill: true, paint: radius(full) },
    ],
  })
>
  <slot></slot>
</span>
