<!--
  <card> — L1 language element (element mode). Generic surface: header/body/footer
  composition, tone-tinted border, status rail, interactive lift.

  Paint (Style product only):
    host sx={[…]} — base surface from component.card tokens (cssNames card,
                    card-foreground), match(variant/padding), compound(interactive/
                    fill/status-rail/tone), when(hover/focus)
    structure     — named header/footer slots with chrome Style; empty slots
                    remain structural (empty:hidden residual not yet on Style)
  Zero CVA-in-props paint maps. Zero scope compound: arrays.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    variant: 'default' | 'outline' = 'default'
    padding: 'md' | 'none' | 'sm' | 'lg' = 'md'
    interactive = false
    'accent-bar': 'off' | 'hover' | 'static' = 'off'
    'status-rail' = false
    fill = false
  }

  const base = style.overflow(clip).radius(lg).border(border).fg(card-foreground).shadow(card-rest).bg(card).padY(4).padX(5)
  const outline = style.bg(transparent)
  const padMd = style.padY(4).padX(5)
  const padNone = style.pad(0)
  const padSm = style.padY(2).padX(3)
  const padLg = style.padY(5).padX(6)
  const headerChrome = style.padX(5).padY(4).border(border-subtle).fontFamily(heading)
  const footerChrome = style.padX(5).padY(4).border(border-subtle)
</scope>

<div sx={[
  base,
  match(variant, {
    default: style.bg(card),
    outline: outline,
  }),
  match(padding, {
    md: padMd,
    none: padNone,
    sm: padSm,
    lg: padLg,
  }),
  compound({ interactive: true }, style.cursor(pointer).shadow(hover).translateY(0)),
  compound({ fill: true }, style.bg(muted).border(border)),
  compound({ tone: primary }, style.border(primary)),
  compound({ tone: info }, style.border(info)),
  compound({ tone: success }, style.border(success)),
  compound({ tone: warning }, style.border(warning)),
  compound({ tone: destructive }, style.border(destructive)),
  compound({ tone: accent }, style.border(accent)),
  compound({ tone: muted }, style.border(border)),
  compound({ status-rail: true }, style.borderS(border-strong).padL(4)),
  compound({ status-rail: true, padding: sm }, style.padL(2)),
  compound({ status-rail: true, padding: md }, style.padL(4)),
  compound({ status-rail: true, padding: lg }, style.padL(5)),
  compound({ status-rail: true, padding: none }, style.padL(0)),
  when({
    hover: style.opacity(100),
    focus: style.outline(2),
  }),
]}>
  <div class="header" sx={headerChrome}>
    <slot name="header"></slot>
  </div>
  <slot></slot>
  <div class="footer" sx={footerChrome}>
    <slot name="footer"></slot>
  </div>
</div>
