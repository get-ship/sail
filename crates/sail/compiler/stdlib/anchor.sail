<!--
  <a> — native <a>. L1 platform link; when `variant` is set, becomes a
  button link. Style value language only.

  Paint (Style product only): variant×tone via flat match + compound, semantic
  hues only (no --tone* attribute channel, no tone-* design tokens). Base
  geometry, Tier-2 chrome (corner-image / control-frame / lift / press), the
  global focus ring, and disabled cursor/opacity stay in contracts.css.
  Hover deepen: solid compounds use bgMix(token, 15) to reproduce the prior
  color-mix(ton, foreground 15%) shift; soft compounds use bgWash(hue, pct)
  for a transparent hue wash (rest + deeper hover).
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

  // Tone-aware compounds for variant=soft — hue wash, not neutral hover-bg.
  const softPrimary = style.bgWash(primary, 15).border(primary).fg(primary).when({ hover: style.bgWash(primary, 25) })
  const softInfo = style.bgWash(info, 15).border(info).fg(info).when({ hover: style.bgWash(info, 25) })
  const softSuccess = style.bgWash(success, 15).border(success).fg(success).when({ hover: style.bgWash(success, 25) })
  const softWarning = style.bgWash(warning, 15).border(warning).fg(warning).when({ hover: style.bgWash(warning, 25) })
  const softDestructive = style.bgWash(destructive, 15).border(destructive).fg(destructive).when({ hover: style.bgWash(destructive, 25) })
  const softAccent = style.bgWash(accent, 15).border(accent).fg(accent).when({ hover: style.bgWash(accent, 25) })
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
