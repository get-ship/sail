<!--
  <empty-state> — L1 centered empty placeholder.
  Style value language only. size/variant/tone ordinary props.
-->

<scope>
  props: {
    variant: 'default' | 'inline' | 'placeholder' | 'connecting' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
  }

  const base = style({ box: padY(16).padX(4), paint: fg(empty-state-foreground).borderWidth(1).border(empty-state-border).radius(lg).bg(empty-state).shadow(xs), text: fontFamily(body).textAlign(center) })
  const sizeMd = style({ box: padY(16).padX(4) })
  const sizeSm = style({ box: padY(7).padX(4) })
  const sizeLg = style({ box: padY(24).padX(6) })
  const variantDefault = style({ paint: borderWidth(1).border(empty-state-border).radius(lg).bg(empty-state).shadow(xs), text: textAlign(center) })
  const variantInline = style({ box: padY(3).padX(4), paint: borderWidth(1).border(dashed).border(empty-state-border).radius(md).bg(empty-state).shadow(none), text: textAlign(start) })
  const variantPlaceholder = style({ box: padY(6), paint: borderWidth(0).bg(transparent).radius(none).shadow(none), text: textAlign(center) })
  const variantConnecting = style({ box: padY(6), paint: borderWidth(0).bg(transparent).radius(none).shadow(none), text: textAlign(center) })
  const icon = style({ box: self(center).size(12), paint: radius(lg).bg(empty-state).fg(empty-state-foreground) })
  const copy = style({ box: self(stretch) })
  const title = style({ paint: fg(empty-state-title), text: fontFamily(heading).fontWeight(semibold).leading(tight).fontSize(md) })
  const desc = style({ box: maxW(panel-ch-w-md), paint: fg(empty-state-foreground), text: leading(loose).fontSize(sm) })
  const actions = style({ box: self(center) })
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:variantLive

>
  style([
    base,
    match(size, {
      md: sizeMd,
      sm: sizeSm,
      lg: sizeLg,
    }),
    match(variant, {
      default: variantDefault,
      inline: variantInline,
      placeholder: variantPlaceholder,
      connecting: variantConnecting,
    }),
    when({
      forced-colors: style({ paint: border(CanvasText) }),
    }),
  ])

  <row inline nowrap align="center" justify="center" hidden="true">
    style([ style({ box: shrink(1) }), 
      icon,
      match(tone, {
        default: style({ paint: bg(empty-state).fg(empty-state-foreground) }),
        primary: style({ paint: bg(empty-state).fg(primary) }),
        info: style({ paint: bg(empty-state).fg(info) }),
        success: style({ paint: bg(empty-state).fg(success) }),
        warning: style({ paint: bg(empty-state).fg(warning) }),
        destructive: style({ paint: bg(empty-state).fg(destructive) }),
        accent: style({ paint: bg(empty-state).fg(accent) }),
        muted: style({ paint: bg(empty-state).fg(empty-state-foreground) }),
      }),
      when({
        empty: style({ box: display(none) }),
        forced-colors: style({ paint: bg(Canvas).fg(CanvasText) }),
      }),
    ])

    <slot name="icon"></slot>
  </row>
  <column align="stretch" gap="1">
    style([ style(copy), style({ box: grow(1).minW(0) }) ])

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
      style([
      desc,
      when({
        empty: style({ box: display(none) }),
      }),
    ])

      <slot></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2">
    style([
    actions,
    when({
      empty: style({ box: display(none) }),
    }),
  ])

    <slot name="action"></slot>
  </row>
</column>
