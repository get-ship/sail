<!--
  <card> — L1 language element (element mode). Generic surface: header/body/footer
  composition, tone-tinted border, status rail, interactive lift.

  Paint (Style product only):
    host sx={[…]} — base surface from component.card tokens (cssNames card,
                    card-foreground) + corner-image/backdrop treatment,
                    match(variant/padding), compound(interactive/fill/
                    status-rail/accent-bar), bound tone border
    structure     — named header/footer slots with chrome Style; unfilled
                    regions collapse via when({ empty: style({ box: display(none) }) }).
  Zero CVA-in-props paint maps. Zero scope compound: arrays.

  Rest border = border token when tone is omitted.
  Semantic border = bound tone. Authors omit tone for rest (never tone="default").
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    variant: 'default' | 'outline' = 'default'
    padding: 'md' | 'none' | 'sm' | 'lg' = 'md'
    interactive = false
    'accent-bar': 'off' | 'hover' | 'static' = 'off'
    'status-rail' = false
    fill = false
  }

  const base = style({ box: position(relative).overflow(clip).padY(4).padX(5), paint: isolate().radius(lg).border(1, border).fg(card-foreground).shadow(card-rest).transition(colors).duration(fast).ease(out).bg(card).backdrop(card-backdrop-filter).borderImage(corner-image) })
  const outline = style({ paint: bg(transparent) })
  const padMd = style({ box: padY(4).padX(5) })
  const padNone = style({ box: pad(0) })
  const padSm = style({ box: padY(2).padX(3) })
  const padLg = style({ box: padY(5).padX(6) })
  const headerChrome = style({ box: mt(-4).mx(-5).mb(3).padX(5).padY(4), paint: borderB(1, border-subtle), text: fontFamily(heading) })
  const footerChrome = style({ box: mt(3).mx(-5).mb(-4).padX(5).padY(4), paint: borderT(1, border-subtle) })
</scope>

<div style([
  base,
  match(variant, {
    default: style({ paint: bg(card) }),
    outline: outline,
  }),
  match(padding, {
    md: padMd,
    none: padNone,
    sm: padSm,
    lg: padLg,
  }),
  compound({ interactive: true }, style({ paint: cursor(pointer).tapHighlight(none) }).when(hover, { paint: border(border-strong).shadow(hover).translateY(-px) }).when(active, { paint: shadow(card-rest).translateY(0).duration(instant) }).when(motion-reduce, { paint: translateY(0) }).when(focus, { paint: outline(2) }).when(forced-colors, when({ focus: style({ paint: outline(CanvasText) }) }))),
  compound({ fill: true }, style({ paint: bg(muted).border(border) })),
  // Bound tone border (omit tone → base border token).
  style({ paint: border(tone) }),
  // Muted special uses border (not pure muted group fill).
  compound({ tone: muted }, style({ paint: border(border) })),
  compound({ status-rail: true }, style({ box: padS(4), paint: borderS(3, border-strong) })),
  compound({ status-rail: true, padding: sm }, style({ box: padS(2) })),
  compound({ status-rail: true, padding: md }, style({ box: padS(4) })),
  compound({ status-rail: true, padding: lg }, style({ box: padS(5) })),
  compound({ status-rail: true, padding: none }, style({ box: padS(0) })),
  compound({ accent-bar: static }, when({
    before: style({ box: position(absolute).left(0).right(0).top(0).h(half), paint: content(empty).bgImage(primary-accent).opacity(100).radiusT(lg).pointerEvents(none).transition(opacity).duration(medium).ease(out) }),
  })),
  compound({ accent-bar: hover }, when({
    before: style({ box: position(absolute).left(0).right(0).top(0).h(half), paint: content(empty).bgImage(primary-accent).opacity(0).radiusT(lg).pointerEvents(none).transition(opacity).duration(medium).ease(out) }),
    hover: when({ before: style({ paint: opacity(100) }) }),
  })),
])>
  <div style([ headerChrome, when({ empty: style({ box: display(none) }) }) ])>
    <slot name="header"></slot>
  </div>
  <slot></slot>
  <div style([ footerChrome, when({ empty: style({ box: display(none) }) }) ])>
    <slot name="footer"></slot>
  </div>
</div>
