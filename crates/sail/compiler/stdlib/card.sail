<!--
  <card> — L1 language element (element mode). Generic surface: header/body/footer
  composition, tone-tinted border, status rail, interactive lift.

  Paint (Style product only):
    host sx={[…]} — base surface from component.card tokens (cssNames card,
                    card-foreground) + corner-image/backdrop treatment,
                    match(variant/padding), compound(interactive/fill/
                    status-rail/tone/accent-bar)
    structure     — named header/footer slots with chrome Style; unfilled
                    regions collapse via when({ empty: style({ box: display(none) }) }).
                    The gate repeats per element because stdlib sources compile
                    standalone — no cross-file Style const sharing exists
                    (documented in compiler/src/stdlib.rs).
  Zero CVA-in-props paint maps. Zero scope compound: arrays.

  Legacy utility-atom spellings this Style paint replaces (pinned by
  sx_host_paint_probe::card_host_paint_bulk_on_host_sx):
    host        backdrop-(--card-backdrop-filter) border-image-(--corner-image)
    rail        border-s-3 border-s-border-strong ps-4
                (branded tones recolored border-s-tone-fg — tone-* attribute
                channel removed; direct per-tone tokens are the product form)
    lift        hover:-translate-y-px active:translate-y-0
                active:duration-instant motion-reduce:translate-y-0
                forced-colors:focus:outline-CanvasText
    accent bar  before:content-empty before:bg-gradient-to-r
                before:from-primary before:to-accent before:h-half
                before:opacity-0 hover:before:opacity-100
                (Style spends the bgImage(primary-accent) token gradient —
                same stops, token-disciplined angle)
    chrome      empty:hidden font-heading font-(--heading-weight)
                border-border-subtle -mt-4 -mx-5 -mb-4
                (heading weight has no Style weight-scale token; family only)
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
  compound({ tone: primary }, style({ paint: border(primary) })),
  compound({ tone: info }, style({ paint: border(info) })),
  compound({ tone: success }, style({ paint: border(success) })),
  compound({ tone: warning }, style({ paint: border(warning) })),
  compound({ tone: destructive }, style({ paint: border(destructive) })),
  compound({ tone: accent }, style({ paint: border(accent) })),
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
