<!--
  <a> — native <a>. Bare (no variant) = text link (sail-base color). With
  variant set = button link. Do NOT default-stamp variant/tone — freeform
  stamping would force every href into button chrome and kill site links.
-->

<scope>
  props: {
    // Defaults exist for the type surface; freeform stamping SKIP variant/tone
    // (see default_prop_attrs) so bare <a href> stays a text link.
    variant: 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'default'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    disabled = false
  }

  // Button-link chrome only when variant is present (attr-gated match arms).
  // display(inline-flex) on base — dual-gate + sail-base both center a[variant]
  // so freeform never leaves the label top-aligned in a block box.
  const base = style.display(inline-flex).appearance(none).fontWeight(medium).leading(none).radius(md).borderWidth(control-stroke).border(solid).border(transparent).minH(control-h-md).padX(3).fontSize(base).select(none).whitespace(nowrap).cursor(pointer).underline(none)
  const vDefault = style.bg(background).border(border).fg(foreground).shadow(xs).when({
    hover: style.bg(muted).border(border-strong),
  })
  const vSolid = style.bg(foreground).border(foreground).fg(background).shadow(xs).when({
    hover: style.opacity(90).shadow(sm),
  })
  const vSoft = style.bg(hover-bg).border(border).fg(foreground).when({
    hover: style.opacity(90),
  })
  const vOutline = style.bg(transparent).border(foreground).fg(foreground).when({
    hover: style.bg(hover-bg),
    disabled: style.bg(transparent),
  })
  const vGhost = style.bg(transparent).border(transparent).fg(foreground).when({
    hover: style.bg(hover-bg),
    disabled: style.bg(transparent),
  })

  // Tone-aware compounds for variant=solid.
  const solidPrimary = style.bg(primary).border(primary).fg(primary-foreground).shadow(xs).when({ hover: style.colorDeepen(primary, 15).shadow(sm) })
  const solidInfo = style.bg(info).border(info).fg(info-foreground).shadow(xs).when({ hover: style.colorDeepen(info, 15).shadow(sm) })
  const solidSuccess = style.bg(success).border(success).fg(success-foreground).shadow(xs).when({ hover: style.colorDeepen(success, 15).shadow(sm) })
  const solidWarning = style.bg(warning).border(warning).fg(warning-foreground).shadow(xs).when({ hover: style.colorDeepen(warning, 15).shadow(sm) })
  const solidDestructive = style.bg(destructive).border(destructive).fg(destructive-foreground).shadow(xs).when({ hover: style.colorDeepen(destructive, 15).shadow(sm) })
  const solidAccent = style.bg(accent).border(accent).fg(accent-foreground).shadow(xs).when({ hover: style.colorDeepen(accent, 15).shadow(sm) })
  const solidMuted = style.bg(muted-foreground).border(muted-foreground).fg(background).shadow(xs).when({ hover: style.colorDeepen(muted-foreground, 15).shadow(sm) })

  // Tone-aware compounds for variant=soft — hue wash, not neutral hover-bg.
  const softPrimary = style.colorMix(primary, 15).border(primary).fg(primary).when({ hover: style.colorMix(primary, 25) })
  const softInfo = style.colorMix(info, 15).border(info).fg(info).when({ hover: style.colorMix(info, 25) })
  const softSuccess = style.colorMix(success, 15).border(success).fg(success).when({ hover: style.colorMix(success, 25) })
  const softWarning = style.colorMix(warning, 15).border(warning).fg(warning).when({ hover: style.colorMix(warning, 25) })
  const softDestructive = style.colorMix(destructive, 15).border(destructive).fg(destructive).when({ hover: style.colorMix(destructive, 25) })
  const softAccent = style.colorMix(accent, 15).border(accent).fg(accent).when({ hover: style.colorMix(accent, 25) })
  const softMuted = style.bg(muted).border(border).fg(muted-foreground).when({ hover: style.opacity(90) })

  // Tone-aware compounds for variant=outline.
  const outlinePrimary = style.border(primary).fg(primary).when({ hover: style.opacity(90) })
  const outlineInfo = style.border(info).fg(info).when({ hover: style.opacity(90) })
  const outlineSuccess = style.border(success).fg(success).when({ hover: style.opacity(90) })
  const outlineWarning = style.border(warning).fg(warning).when({ hover: style.opacity(90) })
  const outlineDestructive = style.border(destructive).fg(destructive).when({ hover: style.opacity(90) })
  const outlineAccent = style.border(accent).fg(accent).when({ hover: style.opacity(90) })
  const outlineMuted = style.border(muted-foreground).fg(muted-foreground).when({ hover: style.opacity(90) })

  // Tone-aware compounds for variant=ghost.
  const ghostPrimary = style.fg(primary).when({ hover: style.opacity(90) })
  const ghostInfo = style.fg(info).when({ hover: style.opacity(90) })
  const ghostSuccess = style.fg(success).when({ hover: style.opacity(90) })
  const ghostWarning = style.fg(warning).when({ hover: style.opacity(90) })
  const ghostDestructive = style.fg(destructive).when({ hover: style.opacity(90) })
  const ghostAccent = style.fg(accent).when({ hover: style.opacity(90) })
  const ghostMuted = style.fg(muted-foreground).when({ hover: style.opacity(90) })
</scope>

<a>
  style([
  base,
  match(variant, {
    default: vDefault,
    solid: vSolid,
    soft: vSoft,
    outline: vOutline,
    ghost: vGhost,
  }),
  // Variant × tone — every rule is attr-gated, no unconditional paint.
  compound({ variant: solid, tone: primary }, solidPrimary),
  compound({ variant: solid, tone: info }, solidInfo),
  compound({ variant: solid, tone: success }, solidSuccess),
  compound({ variant: solid, tone: warning }, solidWarning),
  compound({ variant: solid, tone: destructive }, solidDestructive),
  compound({ variant: solid, tone: accent }, solidAccent),
  compound({ variant: solid, tone: muted }, solidMuted),
  compound({ variant: soft, tone: primary }, softPrimary),
  compound({ variant: soft, tone: info }, softInfo),
  compound({ variant: soft, tone: success }, softSuccess),
  compound({ variant: soft, tone: warning }, softWarning),
  compound({ variant: soft, tone: destructive }, softDestructive),
  compound({ variant: soft, tone: accent }, softAccent),
  compound({ variant: soft, tone: muted }, softMuted),
  compound({ variant: outline, tone: primary }, outlinePrimary),
  compound({ variant: outline, tone: info }, outlineInfo),
  compound({ variant: outline, tone: success }, outlineSuccess),
  compound({ variant: outline, tone: warning }, outlineWarning),
  compound({ variant: outline, tone: destructive }, outlineDestructive),
  compound({ variant: outline, tone: accent }, outlineAccent),
  compound({ variant: outline, tone: muted }, outlineMuted),
  compound({ variant: ghost, tone: primary }, ghostPrimary),
  compound({ variant: ghost, tone: info }, ghostInfo),
  compound({ variant: ghost, tone: success }, ghostSuccess),
  compound({ variant: ghost, tone: warning }, ghostWarning),
  compound({ variant: ghost, tone: destructive }, ghostDestructive),
  compound({ variant: ghost, tone: accent }, ghostAccent),
  compound({ variant: ghost, tone: muted }, ghostMuted),
  when({ disabled: style.opacity(50).cursor(not-allowed) }),
])

  <slot></slot>
</a>
