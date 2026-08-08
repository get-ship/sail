<!--
  <empty-state> — L1 centered empty placeholder.
  Style value language only. size/variant ordinary props.
  Icon rest = empty-state tokens when tone is omitted.
  Semantic paint = bound tone (icon fg).
-->

<scope>
  props: {
    variant: 'default' | 'inline' | 'placeholder' | 'connecting' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
  }

  style base = { box: padY(16).padX(4), paint: fg(empty-state.foreground).borderWidth(1).border(empty-state.border).radius(lg).bg(empty-state).shadow(xs), text: fontFamily(body).textAlign(center) }
  style sizeMd = { box: padY(16).padX(4) }
  style sizeSm = { box: padY(7).padX(4) }
  style sizeLg = { box: padY(24).padX(6) }
  style variantDefault = { paint: borderWidth(1).border(empty-state.border).radius(lg).bg(empty-state).shadow(xs), text: textAlign(center) }
  style variantInline = { box: padY(3).padX(4), paint: borderWidth(1).border(dashed).border(empty-state.border).radius(md).bg(empty-state).shadow(none), text: textAlign(start) }
  // placeholder + connecting share bare (borderless/transparent) chrome.
  style variantBare = { box: padY(6), paint: borderWidth(0).bg(transparent).radius(none).shadow(none), text: textAlign(center) }
  style icon = { box: self(center).size(12), paint: radius(lg).bg(empty-state).fg(empty-state.foreground) }
  style copy = { box: self(stretch) }
  style title = { paint: fg(empty-state.title), text: fontFamily(heading).fontWeight(semibold).leading(tight).fontSize(md) }
  style desc = { box: maxW(panel-ch-w-md), paint: fg(empty-state.foreground), text: leading(loose).fontSize(sm) }
  style actions = { box: self(center) }
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:variantLive
  style(base, {
    when: { forced-colors: { paint: border(CanvasText) } },
    match: {
      size: { md: sizeMd, sm: sizeSm, lg: sizeLg },
      variant: {
        default: variantDefault,
        inline: variantInline,
        placeholder: variantBare,
        connecting: variantBare,
      },
    },
  })
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    hidden="true"
    style(icon, {
      box: shrink(1),
      paint: bg(empty-state).fg(tone),
      when: { empty: { box: display(none) }, forced-colors: { paint: bg(Canvas).fg(CanvasText) } },
      compound: [{ tone: muted, paint: bg(empty-state).fg(empty-state.foreground) }],
    })
  >
    <slot name="icon"></slot>
  </row>
  <column align="stretch" gap="1" style(copy, { box: grow(1).minW(0) })>
    <div style(title, { when: { empty: { box: display(none) } } })>
      <slot name="title"></slot>
    </div>
    <div style(desc, { when: { empty: { box: display(none) } } })>
      <slot></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" style(actions, { when: { empty: { box: display(none) } } })>
    <slot name="action"></slot>
  </row>
</column>
