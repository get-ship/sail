<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.
  Paint (Style product): track + ::before thumb; size via match; tone via bound paths.
  Rest track/border/thumb from component.switch tokens
  (cssNames switch, switch-border, switch-thumb). Checked rest = primary.
  Semantic checked/focus = bound tone. Authors omit tone for rest (never tone="default").
-->

<scope>
  props: {
    checked = false
    disabled = false
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  const base = style({ box: position(relative).display(inline-block).m(0).w(control-h-sm).h(icon-size-md), paint: appearance(none).boxSizing(border).align(middle).cursor(pointer).bg(none).bg(switch).borderWidth(1).border(switch-border).radius(full).tapHighlight(none).transition(colors).duration(fast).ease(out) })
  const sizeMd = style({ box: w(control-h-sm).h(icon-size-md) })
  const sizeSm = style({ box: w(control-h-xs).h(icon-size-sm) })
  const sizeLg = style({ box: w(7).h(icon-size-lg) })
  const thumbMd = style({ box: size(3) })
  const thumbSm = style({ box: size(2_5) })
  const thumbLg = style({ box: size(icon-size-sm) })
</scope>

<input type="checkbox" style([ style({ box: shrink(0) }), 
  base,
  match(size, {
    md: sizeMd,
    sm: sizeSm,
    lg: sizeLg,
  }),
  when({
    hover: style({ paint: bg(hover-bg) }),
    // Checked rest = primary (product on-state when tone omitted).
    checked: style({ paint: bg(primary).border(selected-border).shadow(xs) }),
    focus: style({ paint: outline(solid).outline(2).outline(primary).outlineOffset(ring-offset) }),
    disabled: style({ paint: cursor(not-allowed).bg(switch).border(border).saturate(60).shadow(none) }),
    before: style({ box: position(absolute).top(0).bottom(0).my(auto).left(px).size(3), paint: content(empty).bg(switch-thumb).radius(full).shadow(sm).transition(all).duration(fast).ease(out) }),
  }),
  match(size, {
    md: when({ before: thumbMd }),
    sm: when({ before: thumbSm }),
    lg: when({ before: thumbLg }),
  }),
  when({
    checked: when({
      before: style({ box: left(auto).right(px), paint: bg(primary-foreground) }),
    }),
  }),
  when({
    disabled: when({
      before: style({ paint: shadow(xs) }),
    }),
  }),
  // Bound tone overrides checked track + focus ring when tone is set.
  when({
    checked: style({ paint: bg(tone).border(selected-border).shadow(xs) }),
    focus: style({ paint: outline(tone) }),
  }),
  when({
    checked: when({ before: style({ paint: bg(tone.foreground) }) }),
  }),
  // Muted specials — muted-foreground track / background thumb (not pure group members).
  compound({ tone: muted }, when({
    checked: style({ paint: bg(muted-foreground) }),
    focus: style({ paint: outline(muted-foreground) }),
  })),
  compound({ tone: muted }, when({
    checked: when({ before: style({ paint: bg(background) }) }),
  })),
  when({
    motion-reduce: when({
      before: style({ paint: transition(none) }),
    }),
  }),
  when({
    forced-colors: style({ paint: border(CanvasText) }),
  }),
  when({
    forced-colors: when({
      checked: style({ paint: bg(Highlight) }),
      focus: style({ paint: outline(CanvasText) }),
    }),
  }),
])>
