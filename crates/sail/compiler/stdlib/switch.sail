<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.
  Paint (Style product): track + ::before thumb; size/tone via match.
  Rest track/border/thumb from component.switch tokens
  (cssNames switch, switch-border, switch-thumb); checked/tone stay global/tone.
-->

<scope>
  props: {
    checked = false
    disabled = false
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
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

<input type="checkbox" shrink="0">
  style([
  base,
  match(size, {
    md: sizeMd,
    sm: sizeSm,
    lg: sizeLg,
  }),
  match(tone, {
    default: style({ paint: bg(switch) }),
    primary: style({ paint: bg(switch) }),
    info: style({ paint: bg(switch) }),
    success: style({ paint: bg(switch) }),
    warning: style({ paint: bg(switch) }),
    destructive: style({ paint: bg(switch) }),
    accent: style({ paint: bg(switch) }),
    muted: style({ paint: bg(switch) }),
  }),
  when({
    hover: style({ paint: bg(hover-bg) }),
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
  match(tone, {
    default: when({
      checked: style({ paint: bg(primary) }),
      focus: style({ paint: outline(primary) }),
    }),
    primary: when({
      checked: style({ paint: bg(primary) }),
      focus: style({ paint: outline(primary) }),
    }),
    info: when({
      checked: style({ paint: bg(info) }),
      focus: style({ paint: outline(info) }),
    }),
    success: when({
      checked: style({ paint: bg(success) }),
      focus: style({ paint: outline(success) }),
    }),
    warning: when({
      checked: style({ paint: bg(warning) }),
      focus: style({ paint: outline(warning) }),
    }),
    destructive: when({
      checked: style({ paint: bg(destructive) }),
      focus: style({ paint: outline(destructive) }),
    }),
    accent: when({
      checked: style({ paint: bg(accent) }),
      focus: style({ paint: outline(accent) }),
    }),
    muted: when({
      checked: style({ paint: bg(muted-foreground) }),
      focus: style({ paint: outline(muted-foreground) }),
    }),
  }),
  match(tone, {
    default: when({
      checked: when({ before: style({ paint: bg(primary-foreground) }) }),
    }),
    primary: when({
      checked: when({ before: style({ paint: bg(primary-foreground) }) }),
    }),
    info: when({
      checked: when({ before: style({ paint: bg(info-foreground) }) }),
    }),
    success: when({
      checked: when({ before: style({ paint: bg(success-foreground) }) }),
    }),
    warning: when({
      checked: when({ before: style({ paint: bg(warning-foreground) }) }),
    }),
    destructive: when({
      checked: when({ before: style({ paint: bg(destructive-foreground) }) }),
    }),
    accent: when({
      checked: when({ before: style({ paint: bg(accent-foreground) }) }),
    }),
    muted: when({
      checked: when({ before: style({ paint: bg(background) }) }),
    }),
  }),
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
])
</input>
