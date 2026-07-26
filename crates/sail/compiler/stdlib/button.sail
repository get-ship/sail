<!--
  <button> — native <button>. L1 platform control. Style value language only.

  Paint (Style product only): variant×tone via flat match + compound (the
  card/badge pattern — nested match currently drops host paint), semantic
  hues only (no --tone* attribute channel, no tone-* design tokens). Base
  geometry, Tier-2 chrome (corner-image / control-frame / lift / press), the
  global focus ring, and disabled cursor/opacity stay in contracts.css — the
  Style product has no setter for those channels yet.

  Soft fill uses colorMix(hue, pct) — color-mix(hue pct%, transparent) — so
  soft primary is a real primary tint, not neutral hover-bg. Soft hover
  deepens the wash; solid hover uses opacity.
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
  //   soft    — hue wash surface (colorMix), tone-fg text
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
  compound({ variant: soft, tone: primary }, style.colorMix(primary, 15).border(primary).fg(primary).when({ hover: style.colorMix(primary, 25) })),
  compound({ variant: soft, tone: info }, style.colorMix(info, 15).border(info).fg(info).when({ hover: style.colorMix(info, 25) })),
  compound({ variant: soft, tone: success }, style.colorMix(success, 15).border(success).fg(success).when({ hover: style.colorMix(success, 25) })),
  compound({ variant: soft, tone: warning }, style.colorMix(warning, 15).border(warning).fg(warning).when({ hover: style.colorMix(warning, 25) })),
  compound({ variant: soft, tone: destructive }, style.colorMix(destructive, 15).border(destructive).fg(destructive).when({ hover: style.colorMix(destructive, 25) })),
  compound({ variant: soft, tone: accent }, style.colorMix(accent, 15).border(accent).fg(accent).when({ hover: style.colorMix(accent, 25) })),
  compound({ variant: soft, tone: muted }, style.bg(muted).border(border).fg(muted-foreground)),
  compound({ variant: outline, tone: primary }, style.border(primary).fg(primary)),
  compound({ variant: outline, tone: info }, style.border(info).fg(info)),
  compound({ variant: outline, tone: success }, style.border(success).fg(success)),
  compound({ variant: outline, tone: warning }, style.border(warning).fg(warning)),
  compound({ variant: outline, tone: destructive }, style.border(destructive).fg(destructive)),
  compound({ variant: outline, tone: accent }, style.border(accent).fg(accent)),
  compound({ variant: outline, tone: muted }, style.border(muted-foreground).fg(muted-foreground)),
  compound({ variant: ghost, tone: primary }, style.fg(primary)),
  compound({ variant: ghost, tone: info }, style.fg(info)),
  compound({ variant: ghost, tone: success }, style.fg(success)),
  compound({ variant: ghost, tone: warning }, style.fg(warning)),
  compound({ variant: ghost, tone: destructive }, style.fg(destructive)),
  compound({ variant: ghost, tone: accent }, style.fg(accent)),
  compound({ variant: ghost, tone: muted }, style.fg(muted-foreground)),
]}>
  <slot></slot>
</button>
