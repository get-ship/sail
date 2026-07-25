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

  const base = style.appearance(none).position(relative).shrink(0).box(border).display(inline-block).align(middle).m(0).cursor(pointer).bg(none).bg(switch).borderWidth(1).border(switch-border).radius(full).w(control-h-sm).h(icon-size-md).tapHighlight(none).transition(colors).duration(fast).ease(out)
  const sizeMd = style.w(control-h-sm).h(icon-size-md)
  const sizeSm = style.w(control-h-xs).h(icon-size-sm)
  const sizeLg = style.w(7).h(icon-size-lg)
  const thumbMd = style.size(3)
  const thumbSm = style.size(2_5)
  const thumbLg = style.size(icon-size-sm)
</scope>

<input type="checkbox" role="switch" sx={[
  base,
  match(size, {
    md: sizeMd,
    sm: sizeSm,
    lg: sizeLg,
  }),
  match(tone, {
    default: style.bg(switch),
    primary: style.bg(switch),
    info: style.bg(switch),
    success: style.bg(switch),
    warning: style.bg(switch),
    destructive: style.bg(switch),
    accent: style.bg(switch),
    muted: style.bg(switch),
  }),
  when({
    hover: style.bg(hover-bg),
    checked: style.bg(primary).border(selected-border).shadow(xs),
    focus: style.outline(solid).outline(2).outline(primary).outlineOffset(ring-offset),
    disabled: style.cursor(not-allowed).bg(switch).border(border).saturate(60).shadow(none),
    before: style.content(empty).position(absolute).top(0).bottom(0).my(auto).left(px).bg(switch-thumb).radius(full).shadow(sm).size(3).transition(all).duration(fast).ease(out),
  }),
  match(size, {
    md: when({ before: thumbMd }),
    sm: when({ before: thumbSm }),
    lg: when({ before: thumbLg }),
  }),
  when({
    checked: when({
      before: style.left(auto).right(px).bg(primary-foreground),
    }),
  }),
  when({
    disabled: when({
      before: style.shadow(xs),
    }),
  }),
  match(tone, {
    default: when({
      checked: style.bg(primary),
      focus: style.outline(primary),
    }),
    primary: when({
      checked: style.bg(primary),
      focus: style.outline(primary),
    }),
    info: when({
      checked: style.bg(info),
      focus: style.outline(info),
    }),
    success: when({
      checked: style.bg(success),
      focus: style.outline(success),
    }),
    warning: when({
      checked: style.bg(warning),
      focus: style.outline(warning),
    }),
    destructive: when({
      checked: style.bg(destructive),
      focus: style.outline(destructive),
    }),
    accent: when({
      checked: style.bg(accent),
      focus: style.outline(accent),
    }),
    muted: when({
      checked: style.bg(muted-foreground),
      focus: style.outline(muted-foreground),
    }),
  }),
  match(tone, {
    default: when({
      checked: when({ before: style.bg(primary-foreground) }),
    }),
    primary: when({
      checked: when({ before: style.bg(primary-foreground) }),
    }),
    info: when({
      checked: when({ before: style.bg(info-foreground) }),
    }),
    success: when({
      checked: when({ before: style.bg(success-foreground) }),
    }),
    warning: when({
      checked: when({ before: style.bg(warning-foreground) }),
    }),
    destructive: when({
      checked: when({ before: style.bg(destructive-foreground) }),
    }),
    accent: when({
      checked: when({ before: style.bg(accent-foreground) }),
    }),
    muted: when({
      checked: when({ before: style.bg(background) }),
    }),
  }),
  when({
    motion-reduce: when({
      before: style.transition(none),
    }),
  }),
  when({
    forced-colors: style.border(CanvasText),
  }),
  when({
    forced-colors: when({
      checked: style.bg(Highlight),
      focus: style.outline(CanvasText),
    }),
  }),
]} />
