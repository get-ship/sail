<!--
  <a> — native <a>. Bare (no variant) = text link (sail-base color). With
  variant set = button link. Do NOT default-stamp variant/tone — freeform
  stamping would force every href into button chrome and kill site links.

  Rest paint = component.button tokens when tone is omitted.
  Semantic paint = bound tone group paths. Authors omit tone for rest (never tone="default").
-->

<scope>
  props: {
    // Defaults exist for the type surface; freeform stamping SKIP variant/tone
    // (see default_prop_attrs) so bare <a href> stays a text link.
    variant: 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'default'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
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
  const vSolidTone = style({ paint: bg(tone).border(tone).fg(tone.foreground).shadow(xs) }).when(hover, { paint: opacity(90).shadow(sm) })
  const vSoft = style({ paint: bg(hover-bg).border(button-border).fg(button-foreground) }).when(hover, { paint: opacity(90) })
  const vSoftTone = style({ paint: colorMix(tone, 15).border(tone).fg(tone) }).when(hover, { paint: colorMix(tone, 25) })
  const vOutline = style({ paint: bg(transparent).border(button-foreground).fg(button-foreground) }).when(hover, { paint: bg(hover-bg) }).when(disabled, { paint: bg(transparent) })
  const vOutlineTone = style({ paint: bg(transparent).border(tone).fg(tone) }).when(hover, { paint: bg(hover-bg) }).when(disabled, { paint: bg(transparent) })
  const vGhost = style({ paint: bg(transparent).border(transparent).fg(button-foreground) }).when(hover, { paint: bg(hover-bg) }).when(disabled, { paint: bg(transparent) })
  const vGhostTone = style({ paint: bg(transparent).fg(tone) }).when(hover, { paint: bg(hover-bg) }).when(disabled, { paint: bg(transparent) })
</scope>

<a style([
  base,
  match(variant, {
    default: vDefault,
    solid: vSolid,
    soft: vSoft,
    outline: vOutline,
    ghost: vGhost,
  }),
  // Brand override when tone is set — attr-gated expansion over Tone groups.
  match(variant, {
    solid: vSolidTone,
    soft: vSoftTone,
    outline: vOutlineTone,
    ghost: vGhostTone,
  }),
  // Muted specials — not pure group-member paint for all variants.
  compound({ variant: solid, tone: muted }, style({ paint: bg(muted-foreground).border(muted-foreground).fg(background) })),
  compound({ variant: soft, tone: muted }, style({ paint: bg(muted).border(border).fg(muted-foreground) })),
  compound({ variant: outline, tone: muted }, style({ paint: border(muted-foreground).fg(muted-foreground) })),
  compound({ variant: ghost, tone: muted }, style({ paint: fg(muted-foreground) })),
  when({ disabled: style({ paint: opacity(50).cursor(not-allowed) }) }),
])>
  <slot></slot>
</a>
