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

  const base = style.fontFamily(body).fg(empty-state-foreground).textAlign(center).borderWidth(1).border(empty-state-border).radius(lg).bg(empty-state).shadow(xs).padY(16).padX(4)
  const sizeMd = style.padY(16).padX(4)
  const sizeSm = style.padY(7).padX(4)
  const sizeLg = style.padY(24).padX(6)
  const variantDefault = style.textAlign(center).borderWidth(1).border(empty-state-border).radius(lg).bg(empty-state).shadow(xs)
  const variantInline = style.textAlign(start).borderWidth(1).border(dashed).border(empty-state-border).radius(md).bg(muted).shadow(none).padY(3).padX(4)
  const variantPlaceholder = style.textAlign(center).borderWidth(0).bg(transparent).radius(none).shadow(none).padY(6)
  const variantConnecting = style.textAlign(center).borderWidth(0).bg(transparent).radius(none).shadow(none).padY(6)
  const icon = style.self(center).radius(lg).bg(muted).fg(muted-foreground).size(12)
  const copy = style.self(stretch)
  const title = style.fontFamily(heading).fontWeight(semibold).fg(foreground).leading(tight).fontSize(md)
  const desc = style.leading(loose).fg(muted-foreground).fontSize(sm).maxW(panel-ch-w-md)
  const actions = style.self(center)
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
      forced-colors: style.border(CanvasText),
    }),
  ])

  <row
    inline
    nowrap
    align="center"
    justify="center"
    hidden="true"
    shrink

  >
    style([
      icon,
      match(tone, {
        default: style.bg(muted).fg(muted-foreground),
        primary: style.bg(muted).fg(primary),
        info: style.bg(muted).fg(info),
        success: style.bg(muted).fg(success),
        warning: style.bg(muted).fg(warning),
        destructive: style.bg(muted).fg(destructive),
        accent: style.bg(muted).fg(accent),
        muted: style.bg(muted).fg(muted-foreground),
      }),
      when({
        empty: style.display(none),
        forced-colors: style.bg(Canvas).fg(CanvasText),
      }),
    ])

    <slot name="icon"></slot>
  </row>
  <column align="stretch" gap="1" grow min-w="0">
    style(copy)

    <div>
      style([
      title,
      when({
        empty: style.display(none),
      }),
    ])

      <slot name="title"></slot>
    </div>
    <div>
      style([
      desc,
      when({
        empty: style.display(none),
      }),
    ])

      <slot></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2">
    style([
    actions,
    when({
      empty: style.display(none),
    }),
  ])

    <slot name="action"></slot>
  </row>
</column>
