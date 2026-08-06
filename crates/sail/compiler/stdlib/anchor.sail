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
  const base = style({ box: display(inline-flex).minH(control-h-md).padX(3), paint: appearance(none).radius(md).borderWidth(control-stroke).border(solid).border(transparent).select(none).cursor(pointer), text: fontWeight(medium).leading(none).fontSize(base).whitespace(nowrap).underline(none) })
  // Button-link rest rides shared component.button tokens (same pack as <button>).
  // Bare text links (no variant stamp) stay sail-base primary; branded tones spend globals.
  const vDefault = style({ paint: bg(button).border(button-border).fg(button-foreground).shadow(xs) }).when(hover, { paint: bg(button-hover-bg).border(border-strong) })
  const vSolid = style({ paint: bg(foreground).border(foreground).fg(background).shadow(xs) }).when(hover, { paint: opacity(90).shadow(sm) })
  const vSoft = style({ paint: bg(hover-bg).border(button-border).fg(button-foreground) }).when(hover, { paint: opacity(90) })
  const vOutline = style({ paint: bg(transparent).border(button-foreground).fg(button-foreground) }).when(hover, { paint: bg(hover-bg) }).when(disabled, { paint: bg(transparent) })
  const vGhost = style({ paint: bg(transparent).border(transparent).fg(button-foreground) }).when(hover, { paint: bg(hover-bg) }).when(disabled, { paint: bg(transparent) })

  // Tone-aware compounds for variant=solid.
  const solidPrimary = style({ paint: bg(primary).border(primary).fg(primary-foreground).shadow(xs) }).when(hover, { paint: colorDeepen(primary, 15).shadow(sm) })
  const solidInfo = style({ paint: bg(info).border(info).fg(info-foreground).shadow(xs) }).when(hover, { paint: colorDeepen(info, 15).shadow(sm) })
  const solidSuccess = style({ paint: bg(success).border(success).fg(success-foreground).shadow(xs) }).when(hover, { paint: colorDeepen(success, 15).shadow(sm) })
  const solidWarning = style({ paint: bg(warning).border(warning).fg(warning-foreground).shadow(xs) }).when(hover, { paint: colorDeepen(warning, 15).shadow(sm) })
  const solidDestructive = style({ paint: bg(destructive).border(destructive).fg(destructive-foreground).shadow(xs) }).when(hover, { paint: colorDeepen(destructive, 15).shadow(sm) })
  const solidAccent = style({ paint: bg(accent).border(accent).fg(accent-foreground).shadow(xs) }).when(hover, { paint: colorDeepen(accent, 15).shadow(sm) })
  const solidMuted = style({ paint: bg(muted-foreground).border(muted-foreground).fg(background).shadow(xs) }).when(hover, { paint: colorDeepen(muted-foreground, 15).shadow(sm) })

  // Tone-aware compounds for variant=soft — hue wash, not neutral hover-bg.
  const softPrimary = style({ paint: colorMix(primary, 15).border(primary).fg(primary) }).when(hover, { paint: colorMix(primary, 25) })
  const softInfo = style({ paint: colorMix(info, 15).border(info).fg(info) }).when(hover, { paint: colorMix(info, 25) })
  const softSuccess = style({ paint: colorMix(success, 15).border(success).fg(success) }).when(hover, { paint: colorMix(success, 25) })
  const softWarning = style({ paint: colorMix(warning, 15).border(warning).fg(warning) }).when(hover, { paint: colorMix(warning, 25) })
  const softDestructive = style({ paint: colorMix(destructive, 15).border(destructive).fg(destructive) }).when(hover, { paint: colorMix(destructive, 25) })
  const softAccent = style({ paint: colorMix(accent, 15).border(accent).fg(accent) }).when(hover, { paint: colorMix(accent, 25) })
  const softMuted = style({ paint: bg(muted).border(border).fg(muted-foreground) }).when(hover, { paint: opacity(90) })

  // Tone-aware compounds for variant=outline.
  const outlinePrimary = style({ paint: border(primary).fg(primary) }).when(hover, { paint: opacity(90) })
  const outlineInfo = style({ paint: border(info).fg(info) }).when(hover, { paint: opacity(90) })
  const outlineSuccess = style({ paint: border(success).fg(success) }).when(hover, { paint: opacity(90) })
  const outlineWarning = style({ paint: border(warning).fg(warning) }).when(hover, { paint: opacity(90) })
  const outlineDestructive = style({ paint: border(destructive).fg(destructive) }).when(hover, { paint: opacity(90) })
  const outlineAccent = style({ paint: border(accent).fg(accent) }).when(hover, { paint: opacity(90) })
  const outlineMuted = style({ paint: border(muted-foreground).fg(muted-foreground) }).when(hover, { paint: opacity(90) })

  // Tone-aware compounds for variant=ghost.
  const ghostPrimary = style({ paint: fg(primary) }).when(hover, { paint: opacity(90) })
  const ghostInfo = style({ paint: fg(info) }).when(hover, { paint: opacity(90) })
  const ghostSuccess = style({ paint: fg(success) }).when(hover, { paint: opacity(90) })
  const ghostWarning = style({ paint: fg(warning) }).when(hover, { paint: opacity(90) })
  const ghostDestructive = style({ paint: fg(destructive) }).when(hover, { paint: opacity(90) })
  const ghostAccent = style({ paint: fg(accent) }).when(hover, { paint: opacity(90) })
  const ghostMuted = style({ paint: fg(muted-foreground) }).when(hover, { paint: opacity(90) })
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
  when({ disabled: style({ paint: opacity(50).cursor(not-allowed) }) }),
])

  <slot></slot>
</a>
