<!--
  <a> — native <a>. L1 platform link; when `variant` is set, becomes a
  button link. Style value language only.

  Paint (Style product only): variant×tone via flat match + compound, semantic
  + tone-ramp tokens (no --tone* attribute channel). Base geometry, Tier-2
  chrome (corner-image / control-frame / lift / press), the global focus ring,
  and disabled cursor/opacity stay in contracts.css.
  Hover deepen: the solid compounds use bgMix(token, 15) to reproduce the
  prior color-mix(ton, foreground 15%) shift; soft/outline/ghost and the
  untoned default variant use the badge/chip opacity(90) idiom because bgMix
  only mixes toward foreground.
-->

<scope>
  props: {
    variant: 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'default'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    disabled = false
  }

  // Untoned variant paint — the match(variant) arm is the default-tone path.
  // Compounds below override per tone.
  const vDefault = style.bg(background).border(border).fg(foreground).shadow(xs).when({
    hover: style.bg(muted).border(border-strong),
  })
  const vSolid = style.bg(foreground).border(foreground).fg(background).shadow(xs).when({
    hover: style.opacity(90).shadow(sm),
  })
  const vSoft = style.bg(hover-bg).border(border).fg(foreground).when({
    hover: style.opacity(90),
  })
  const vOutline = style.border(foreground).fg(foreground).when({
    hover: style.bg(hover-bg),
    disabled: style.bg(transparent),
  })
  const vGhost = style.fg(foreground).when({
    hover: style.bg(hover-bg),
    disabled: style.bg(transparent),
  })

  // Tone-aware compounds for variant=solid.
  const solidPrimary = style.bg(primary).border(primary).fg(primary-foreground).shadow(xs).when({ hover: style.bgMix(primary, 15).shadow(sm) })
  const solidInfo = style.bg(info).border(info).fg(info-foreground).shadow(xs).when({ hover: style.bgMix(info, 15).shadow(sm) })
  const solidSuccess = style.bg(success).border(success).fg(success-foreground).shadow(xs).when({ hover: style.bgMix(success, 15).shadow(sm) })
  const solidWarning = style.bg(warning).border(warning).fg(warning-foreground).shadow(xs).when({ hover: style.bgMix(warning, 15).shadow(sm) })
  const solidDestructive = style.bg(destructive).border(destructive).fg(destructive-foreground).shadow(xs).when({ hover: style.bgMix(destructive, 15).shadow(sm) })
  const solidAccent = style.bg(accent).border(accent).fg(accent-foreground).shadow(xs).when({ hover: style.bgMix(accent, 15).shadow(sm) })
  const solidMuted = style.bg(muted-foreground).border(muted-foreground).fg(background).shadow(xs).when({ hover: style.bgMix(muted-foreground, 15).shadow(sm) })

  // Tone-aware compounds for variant=soft.
  const softPrimary = style.bg(tone-primary-bg).border(tone-primary-border).fg(primary).when({ hover: style.opacity(90) })
  const softInfo = style.bg(tone-info-bg).border(tone-info-border).fg(info).when({ hover: style.opacity(90) })
  const softSuccess = style.bg(tone-success-bg).border(tone-success-border).fg(tone-success-fg).when({ hover: style.opacity(90) })
  const softWarning = style.bg(tone-warning-bg).border(tone-warning-border).fg(tone-warning-fg).when({ hover: style.opacity(90) })
  const softDestructive = style.bg(tone-destructive-bg).border(tone-destructive-border).fg(destructive).when({ hover: style.opacity(90) })
  const softAccent = style.bg(tone-accent-bg).border(tone-accent-border).fg(accent).when({ hover: style.opacity(90) })
  const softMuted = style.bg(tone-muted-bg).border(tone-muted-border).fg(muted-foreground).when({ hover: style.opacity(90) })

  // Tone-aware compounds for variant=outline.
  const outlinePrimary = style.border(primary).fg(primary).when({ hover: style.opacity(90) })
  const outlineInfo = style.border(info).fg(info).when({ hover: style.opacity(90) })
  const outlineSuccess = style.border(success).fg(tone-success-fg).when({ hover: style.opacity(90) })
  const outlineWarning = style.border(warning).fg(tone-warning-fg).when({ hover: style.opacity(90) })
  const outlineDestructive = style.border(destructive).fg(destructive).when({ hover: style.opacity(90) })
  const outlineAccent = style.border(accent).fg(accent).when({ hover: style.opacity(90) })
  const outlineMuted = style.border(muted-foreground).fg(muted-foreground).when({ hover: style.opacity(90) })

  // Tone-aware compounds for variant=ghost.
  const ghostPrimary = style.fg(primary).when({ hover: style.opacity(90) })
  const ghostInfo = style.fg(info).when({ hover: style.opacity(90) })
  const ghostSuccess = style.fg(tone-success-fg).when({ hover: style.opacity(90) })
  const ghostWarning = style.fg(tone-warning-fg).when({ hover: style.opacity(90) })
  const ghostDestructive = style.fg(destructive).when({ hover: style.opacity(90) })
  const ghostAccent = style.fg(accent).when({ hover: style.opacity(90) })
  const ghostMuted = style.fg(muted-foreground).when({ hover: style.opacity(90) })
</scope>

<a sx={[
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
]}>
  <slot></slot>
</a>
