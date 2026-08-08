<!--
  <card> — L1 language element (element mode). Generic surface: header/body/footer
  composition, tone-tinted border, status rail, interactive lift.

  Paint (Style product only):
    host sx={[…]} — base surface from component.card tokens (cssNames card,
                    card.foreground) + corner-image/backdrop treatment,
                    style({ match: { variant: { padding: { } } } }), style({ compound: [ { interactive: fill, status-rail: accent-bar } ] }), bound tone border
    structure     — named header/footer slots with chrome Style; unfilled
                    regions collapse via style({ when: { empty: { box: display(none) } } }).
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

  style base = { box: position(relative).overflow(clip).padY(4).padX(5), paint: isolate().radius(lg).border(1, border).fg(card.foreground).shadow(card-rest).transition(colors).duration(fast).ease(out).bg(card).backdrop(card-backdrop-filter).borderImage(corner-image) }
  style outline = { paint: bg(transparent) }
  style padMd = { box: padY(4).padX(5) }
  style padNone = { box: pad(0) }
  style padSm = { box: padY(2).padX(3) }
  style padLg = { box: padY(5).padX(6) }
  style headerChrome = { box: mt(-4).mx(-5).mb(3).padX(5).padY(4), paint: borderB(1, border-subtle), text: fontFamily(heading) }
  style footerChrome = { box: mt(3).mx(-5).mb(-4).padX(5).padY(4), paint: borderT(1, border-subtle) }
</scope>

<div style(base, { paint: border(tone), match: { variant: { default: { paint: bg(card) }, outline: outline }, padding: { md: padMd, none: padNone, sm: padSm, lg: padLg } }, compound: [ { interactive: true, paint: cursor(pointer).tapHighlight(none), when: { hover: { paint: border(border-strong).shadow(hover).translateY(-px) }, active: { paint: shadow(card-rest).translateY(0).duration(instant) }, motion-reduce: { paint: translateY(0) }, focus: { paint: outline(2) }, forced-colors: { when: { focus: { paint: outline(CanvasText) } } } } }, { fill: true, paint: bg(muted).border(border) }, { tone: muted, paint: border(border) }, { status-rail: true, box: padS(4), paint: borderS(3, border-strong) }, { padding: sm, status-rail: true, box: padS(2) }, { padding: md, status-rail: true, box: padS(4) }, { padding: lg, status-rail: true, box: padS(5) }, { padding: none, status-rail: true, box: padS(0) }, { accent-bar: static, when: { before: { box: position(absolute).left(0).right(0).top(0).h(half), paint: content(empty).bgImage(primary-accent).opacity(100).radiusT(lg).pointerEvents(none).transition(opacity).duration(medium).ease(out) } } }, { accent-bar: hover, when: { before: { box: position(absolute).left(0).right(0).top(0).h(half), paint: content(empty).bgImage(primary-accent).opacity(0).radiusT(lg).pointerEvents(none).transition(opacity).duration(medium).ease(out) }, hover: { when: { before: { paint: opacity(100) } } } } } ] })>
  <div style(headerChrome, { when: { empty: { box: display(none) } } })>
    <slot name="header"></slot>
  </div>
  <slot></slot>
  <div style(footerChrome, { when: { empty: { box: display(none) } } })>
    <slot name="footer"></slot>
  </div>
</div>
