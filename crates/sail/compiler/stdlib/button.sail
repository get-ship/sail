<!--
  <button> — native <button>. L1 platform control. Style value language only.

  Paint (Style product only): variant×tone via flat match + compound (the
  card/badge pattern — nested match currently drops host paint), semantic
  + tone-ramp tokens (no --tone* attribute channel). Base geometry, Tier-2
  chrome (corner-image / control-frame / lift / press), the global focus
  ring, and disabled cursor/opacity stay in contracts.css — the Style
  product has no setter for those channels yet. Hover deepen uses the
  badge/chip idiom (opacity) — no color-mix setter exists (model gap).

  Replaces the contracts BUTTON_SKIN_CSS variant rules that read the
  [tone=] → --tone* channel (the 3e8d3ac5 restoration). tone is an
  ordinary prop with a closed union; paint is Style match/compound.
-->

<scope>
  props: {
    variant: 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'default'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    disabled = false
  }

  // Size ladder — mirrors the contracts [size] geometry (control-h / text /
  // space tokens, no --size-* channel).
  const sizeXs = style.minH(control-h-2xs).padX(2).fontSize(xs)
  const sizeSm = style.minH(control-h-sm).padX(2_5).fontSize(sm)
  const sizeMd = style.minH(control-h-md).padX(3).fontSize(base)
  const sizeLg = style.minH(control-h-lg).padX(4).fontSize(md)

  // NOTE: `//` comments inside the sx list silently drop compound entries
  // (parser gap) — group notes live here instead. Compound groups:
  //   solid   — the tone fill, contrast text on top
  //   soft    — the tinted surface, tone-fg text
  //   outline — tone border + tone-fg text on the page surface
  //   ghost   — tone-fg text only

  // Variant paint at tone=default — the untoned path (a bare <button
  // variant=solid> carries no tone attribute, so the match arm IS the
  // default-tone paint; compounds below override per tone).
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
</scope>

<button sx={[
  match(variant, {
    default: vDefault,
    solid: vSolid,
    soft: vSoft,
    outline: vOutline,
    ghost: vGhost,
  }),
  match(size, {
    xs: sizeXs,
    sm: sizeSm,
    md: sizeMd,
    lg: sizeLg,
  }),
  compound({ variant: solid, tone: primary }, style.bg(primary).border(primary).fg(primary-foreground)),
  compound({ variant: solid, tone: info }, style.bg(info).border(info).fg(info-foreground)),
  compound({ variant: solid, tone: success }, style.bg(success).border(success).fg(success-foreground)),
  compound({ variant: solid, tone: warning }, style.bg(warning).border(warning).fg(warning-foreground)),
  compound({ variant: solid, tone: destructive }, style.bg(destructive).border(destructive).fg(destructive-foreground)),
  compound({ variant: solid, tone: accent }, style.bg(accent).border(accent).fg(accent-foreground)),
  compound({ variant: solid, tone: muted }, style.bg(muted-foreground).border(muted-foreground).fg(background)),
  compound({ variant: soft, tone: primary }, style.bg(tone-primary-bg).border(tone-primary-border).fg(primary)),
  compound({ variant: soft, tone: info }, style.bg(tone-info-bg).border(tone-info-border).fg(info)),
  compound({ variant: soft, tone: success }, style.bg(tone-success-bg).border(tone-success-border).fg(tone-success-fg)),
  compound({ variant: soft, tone: warning }, style.bg(tone-warning-bg).border(tone-warning-border).fg(tone-warning-fg)),
  compound({ variant: soft, tone: destructive }, style.bg(tone-destructive-bg).border(tone-destructive-border).fg(destructive)),
  compound({ variant: soft, tone: accent }, style.bg(tone-accent-bg).border(tone-accent-border).fg(accent)),
  compound({ variant: soft, tone: muted }, style.bg(tone-muted-bg).border(tone-muted-border).fg(muted-foreground)),
  compound({ variant: outline, tone: primary }, style.border(primary).fg(primary)),
  compound({ variant: outline, tone: info }, style.border(info).fg(info)),
  compound({ variant: outline, tone: success }, style.border(success).fg(tone-success-fg)),
  compound({ variant: outline, tone: warning }, style.border(warning).fg(tone-warning-fg)),
  compound({ variant: outline, tone: destructive }, style.border(destructive).fg(destructive)),
  compound({ variant: outline, tone: accent }, style.border(accent).fg(accent)),
  compound({ variant: outline, tone: muted }, style.border(muted-foreground).fg(muted-foreground)),
  compound({ variant: ghost, tone: primary }, style.fg(primary)),
  compound({ variant: ghost, tone: info }, style.fg(info)),
  compound({ variant: ghost, tone: success }, style.fg(tone-success-fg)),
  compound({ variant: ghost, tone: warning }, style.fg(tone-warning-fg)),
  compound({ variant: ghost, tone: destructive }, style.fg(destructive)),
  compound({ variant: ghost, tone: accent }, style.fg(accent)),
  compound({ variant: ghost, tone: muted }, style.fg(muted-foreground)),
]}>
  <slot></slot>
</button>
