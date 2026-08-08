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
  style base = { box: display(inline-flex).minH(control-h-md).padX(3), paint: appearance(none).radius(md).borderWidth(control-stroke).border(solid).border(transparent).select(none).cursor(pointer), text: fontWeight(medium).leading(none).fontSize(base).whitespace(nowrap).underline(none) }
  // Button-link rest rides shared component.button tokens (same pack as <button>).
  // Bare text links (no variant stamp) stay sail-base primary; branded tones spend globals.
  style vDefault = { paint: bg(button).border(button.border).fg(button.foreground).shadow(xs), when: { hover: { paint: bg(hover-bg).border(border-strong) } } }
  style vSolid = { paint: bg(foreground).border(foreground).fg(background).shadow(xs), when: { hover: { paint: opacity(90).shadow(sm) } } }
  style vSolidTone = { paint: bg(tone).border(tone).fg(tone.foreground).shadow(xs), when: { hover: { paint: opacity(90).shadow(sm) } } }
  style vSoft = { paint: bg(hover-bg).border(button.border).fg(button.foreground), when: { hover: { paint: opacity(90) } } }
  style vSoftTone = { paint: colorMix(tone, 15).border(tone).fg(tone), when: { hover: { paint: colorMix(tone, 25) } } }
  style vOutline = { paint: bg(transparent).border(button.foreground).fg(button.foreground), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
  style vOutlineTone = { paint: bg(transparent).border(tone).fg(tone), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
  style vGhost = { paint: bg(transparent).border(transparent).fg(button.foreground), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
  style vGhostTone = { paint: bg(transparent).fg(tone), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } }
</scope>

<a style(base, { when: { disabled: { paint: opacity(50).cursor(not-allowed) } }, match: { variant: { default: vDefault, solid: style(vSolid, vSolidTone), soft: style(vSoft, vSoftTone), outline: style(vOutline, vOutlineTone), ghost: style(vGhost, vGhostTone) } }, compound: [ { tone: muted, variant: solid, paint: bg(muted-foreground).border(muted-foreground).fg(background) }, { tone: muted, variant: soft, paint: bg(muted).border(border).fg(muted-foreground) }, { tone: muted, variant: outline, paint: border(muted-foreground).fg(muted-foreground) }, { tone: muted, variant: ghost, paint: fg(muted-foreground) } ] })>
  <slot></slot>
</a>
