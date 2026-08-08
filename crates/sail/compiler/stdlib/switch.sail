<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.
  Paint (Style product): track + ::before thumb; size via match; tone via bound paths.
  Rest track/border/thumb from component.switch tokens
  (cssNames switch, switch.border, switch.thumb). Checked rest = primary.
  Semantic checked/focus = bound tone. Authors omit tone for rest (never tone="default").
-->

<scope>
  props: {
    checked = false
    disabled = false
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  const base = style({ box: position(relative).display(inline-block).m(0).w(control-h-sm).h(icon-size-md), paint: appearance(none).boxSizing(border).align(middle).cursor(pointer).bg(none).bg(switch).borderWidth(1).border(switch.border).radius(full).tapHighlight(none).transition(colors).duration(fast).ease(out) })
  const sizeMd = style({ box: w(control-h-sm).h(icon-size-md) })
  const sizeSm = style({ box: w(control-h-xs).h(icon-size-sm) })
  const sizeLg = style({ box: w(7).h(icon-size-lg) })
  const thumbMd = style({ box: size(3) })
  const thumbSm = style({ box: size(2_5) })
  const thumbLg = style({ box: size(icon-size-sm) })
</scope>

<input type="checkbox" style(base, { box: shrink(0), when: { hover: { paint: bg(hover-bg) }, checked: { paint: bg(primary).border(selected-border).shadow(xs) }, focus: { paint: outline(solid).outline(2).outline(primary).outlineOffset(ring-offset) }, disabled: { paint: cursor(not-allowed).bg(switch).border(border).saturate(60).shadow(none) }, before: { box: position(absolute).top(0).bottom(0).my(auto).left(px).size(3), paint: content(empty).bg(switch.thumb).radius(full).shadow(sm).transition(all).duration(fast).ease(out) }, checked: { when: { before: { box: left(auto).right(px), paint: bg(primary-foreground) } } }, disabled: { when: { before: { paint: shadow(xs) } } }, checked: { paint: bg(tone).border(selected-border).shadow(xs) }, focus: { paint: outline(tone) }, checked: { when: { before: { paint: bg(tone.foreground) } } }, motion-reduce: { when: { before: { paint: transition(none) } } }, forced-colors: { paint: border(CanvasText) }, forced-colors: { when: { checked: { paint: bg(Highlight) }, focus: { paint: outline(CanvasText) } } } }, match: { size: { md: style(sizeMd, { when: { before: thumbMd } }), sm: style(sizeSm, { when: { before: thumbSm } }), lg: style(sizeLg, { when: { before: thumbLg } }) } }, compound: [ { tone: muted, when: { checked: { paint: bg(muted-foreground) }, focus: { paint: outline(muted-foreground) } } }, { tone: muted, when: { checked: { when: { before: { paint: bg(background) } } } } } ] })>
