<!--
  <button> — native <button>. L1 platform control. Style value language only.

  Paint + size live HERE. Flex geometry (inline-flex / align-items / gap) and
  Tier-2 chrome (control-frame / corner-image / lift / press) that Style already
  owns live here too. Focus ring + disabled opacity stay on the shared document
  baseline — not a parallel BUTTON_SKIN_CSS residual.

  Rest paint = component.button tokens when tone is omitted.
  Semantic paint = bound tone group paths. Authors omit tone for rest (never tone="default").
-->

<scope>
  props: {
    variant: 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'default'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    disabled = false
  }

  // border(solid) kills UA button outset; without it freeform demos look like
  // native OS chrome instead of design-system controls.
  //
  // No base bg(transparent): that class is a cascade landmine (F7). Unlayered
  // product `.sx-bgtransparent` beats layered solid skins and empties primary
  // fills. Variants own background; ghost is the only transparent fill.
  const base = style({ box: display(inline-flex).position(relative), paint: appearance(none).radius(md).borderWidth(control-stroke).border(solid).border(transparent).borderImage(corner-image).isolate().controlFrame().select(none).cursor(pointer), text: fontWeight(medium).leading(none).whitespace(nowrap), when: { hover: { paint: controlLift() }, active: { paint: press() }, disabled: { paint: cursor(not-allowed).opacity(50) } } })

  const sizeXs = style({ box: minH(control-h-2xs).padX(2), text: fontSize(xs) })
  const sizeSm = style({ box: minH(control-h-sm).padX(2_5), text: fontSize(sm) })
  const sizeMd = style({ box: minH(control-h-md).padX(3), text: fontSize(base) })
  const sizeLg = style({ box: minH(control-h-lg).padX(4), text: fontSize(md) })

  // Default rest rides component.button tokens (global fallbacks in the pack).
  // Branded solid/soft/outline tones spend the bound Tone table.
  const vDefault = style({ paint: bg(button).border(button.border).fg(button.foreground).shadow(xs), when: { hover: { paint: bg(hover-bg).border(border-strong) } } })
  // Solid without tone: inverted fg/bg. Solid + tone: bound group paint.
  const vSolid = style({ paint: bg(foreground).border(foreground).fg(background).shadow(xs), when: { hover: { paint: opacity(90).shadow(sm) } } })
  const vSolidTone = style({ paint: bg(tone).border(tone).fg(tone.foreground).shadow(xs), when: { hover: { paint: opacity(90).shadow(sm) } } })
  const vSoft = style({ paint: bg(hover-bg).border(button.border).fg(button.foreground), when: { hover: { paint: opacity(90) } } })
  const vSoftTone = style({ paint: colorMix(tone, 15).border(tone).fg(tone), when: { hover: { paint: colorMix(tone, 25) } } })
  // Outline rest MUST set bg(transparent). Without it, UA buttonface (light gray)
  // shows through on dark themes even with appearance(none) — looks unthemed.
  // Scoped to [variant=outline] only — not base — so F7 cascade landmine stays dead.
  const vOutline = style({ paint: bg(transparent).border(button.foreground).fg(button.foreground), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } })
  const vOutlineTone = style({ paint: bg(transparent).border(tone).fg(tone), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } })
  const vGhost = style({ paint: bg(transparent).fg(button.foreground), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } })
  const vGhostTone = style({ paint: bg(transparent).fg(tone), when: { hover: { paint: bg(hover-bg) }, disabled: { paint: bg(transparent) } } })
</scope>

<button style(base, { match: { variant: { default: vDefault, solid: style(vSolid, vSolidTone), soft: style(vSoft, vSoftTone), outline: style(vOutline, vOutlineTone), ghost: style(vGhost, vGhostTone) }, size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [ { tone: muted, variant: solid, paint: bg(muted-foreground).border(muted-foreground).fg(background) }, { tone: muted, variant: soft, paint: bg(muted).border(border-strong).fg(foreground) }, { tone: muted, variant: outline, paint: border(muted-foreground).fg(muted-foreground) }, { tone: muted, variant: ghost, paint: fg(muted-foreground) } ] })>
  <slot></slot>
</button>
