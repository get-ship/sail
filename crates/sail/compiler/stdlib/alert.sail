<!--
  <alert> — L1 inline status banner. Style value language only.
  Stripe via when(before) + match(tone) semantic tokens.
-->

<scope>
  props: {
    tone: 'default' | 'muted' | 'info' | 'success' | 'warning' | 'destructive' = 'default'
    dismissible = false
  
    onDismiss: () => void
}

  const base = style({ box: position(relative).overflow(hidden).pad(3), paint: group().borderWidth(1).border(alert-border).radius(md).bg(alert).fg(alert-foreground).shadow(xs), text: fontFamily(body).fontSize(base).leading(normal) })
  const stripe = style({ box: position(absolute).left(0).top(0).bottom(0).w(1), paint: content(empty).bg(alert-stripe) })
  // Icon rest samples alert surface; tone compounds still spend semantic palette for glyph color.
  const ico = style({ box: size(control-h-xs), paint: radius(md).bg(alert).fg(alert-stripe) })
  const title = style({ text: fontWeight(semibold) })
  const body = style({ paint: fg(alert-stripe) })
  const actions = style({ box: mt(2) })
  const close = style({ box: display(none).size(control-h-xs).self(start).pad(0), paint: borderWidth(0).radius(sm).bg(transparent).fg(alert-stripe).cursor(pointer).transition(colors).duration(fast).ease(out) })
  const xmark = style({ box: display(block).size(icon-size-sm), paint: stroke(currentColor).fill(none) })
</scope>

<row
  nowrap
  align="start"
  gap="3"
  use:liveRegion
  use:dismissControl

>
  style([
    base,
    when({
      before: stripe,
      forced-colors: style({ paint: border(CanvasText) }),
    }),
    match(tone, {
      default: when({ before: style({ paint: bg(alert-stripe) }) }),
      muted: when({ before: style({ paint: bg(alert-stripe) }) }),
      info: when({ before: style({ paint: bg(info) }) }),
      success: when({ before: style({ paint: bg(success) }) }),
      warning: when({ before: style({ paint: bg(warning) }) }),
      destructive: when({ before: style({ paint: bg(destructive) }) }),
    }),
    when({
      forced-colors: when({
        before: style({ paint: bg(CanvasText) }),
      }),
    }),
  ])

  <row inline nowrap align="center" justify="center">
    style([ style({ box: shrink(1) }), 
    ico,
    match(tone, {
      default: style({ paint: bg(alert).fg(alert-stripe) }),
      muted: style({ paint: bg(alert).fg(alert-stripe) }),
      info: style({ paint: bg(alert).fg(info) }),
      success: style({ paint: bg(alert).fg(success) }),
      warning: style({ paint: bg(alert).fg(warning) }),
      destructive: style({ paint: bg(alert).fg(destructive) }),
    }),
    when({
      empty: style({ box: display(none) }),
    }),
  ])

    <slot name="icon"></slot>
  </row>
  <column gap="1">
    style({ box: grow(1).minW(0) })

    <div>
      style([
      title,
      when({
        empty: style({ box: display(none) }),
      }),
    ])

      <slot name="title"></slot>
    </div>
    <div>
      style(body)

      <slot></slot>
    </div>
    <row align="center" gap="2">
      style([
      actions,
      when({
        empty: style({ box: display(none) }),
      }),
    ])

      <slot name="actions"></slot>
    </row>
  </column>
  <button data-dismiss type="button" anatomy label="Dismiss">
    style([ style({ box: shrink(1) }), 
      close,
      when({
        group-dismissible: style({ box: display(inline-block) }),
        hover: style({ paint: bg(alert).fg(alert-foreground) }),
        focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(2) }),
      }),
    ])

    <svg viewBox="0 0 10 10" hidden="true" stroke-width="1.75" stroke-linecap="round">
      style(xmark)

      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</row>
