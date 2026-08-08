<!--
  <empty-state> — L1 centered empty placeholder.
  Style value language only. size/variant ordinary props.
  Icon rest = empty-state tokens when tone is omitted.
  Semantic paint = bound tone (icon fg). Authors omit tone for rest (never tone="default").
-->

<scope>
  props: {
    variant: 'default' | 'inline' | 'placeholder' | 'connecting' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
  }

  const base = style({ box: padY(16).padX(4), paint: fg(empty-state.foreground).borderWidth(1).border(empty-state.border).radius(lg).bg(empty-state).shadow(xs), text: fontFamily(body).textAlign(center) })
  const sizeMd = style({ box: padY(16).padX(4) })
  const sizeSm = style({ box: padY(7).padX(4) })
  const sizeLg = style({ box: padY(24).padX(6) })
  const variantDefault = style({ paint: borderWidth(1).border(empty-state.border).radius(lg).bg(empty-state).shadow(xs), text: textAlign(center) })
  const variantInline = style({ box: padY(3).padX(4), paint: borderWidth(1).border(dashed).border(empty-state.border).radius(md).bg(empty-state).shadow(none), text: textAlign(start) })
  const variantPlaceholder = style({ box: padY(6), paint: borderWidth(0).bg(transparent).radius(none).shadow(none), text: textAlign(center) })
  const variantConnecting = style({ box: padY(6), paint: borderWidth(0).bg(transparent).radius(none).shadow(none), text: textAlign(center) })
  const icon = style({ box: self(center).size(12), paint: radius(lg).bg(empty-state).fg(empty-state.foreground) })
  const copy = style({ box: self(stretch) })
  const title = style({ paint: fg(empty-state.title), text: fontFamily(heading).fontWeight(semibold).leading(tight).fontSize(md) })
  const desc = style({ box: maxW(panel-ch-w-md), paint: fg(empty-state.foreground), text: leading(loose).fontSize(sm) })
  const actions = style({ box: self(center) })
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:variantLive style(base, { when: { forced-colors: { paint: border(CanvasText) } }, match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, variant: { default: variantDefault, inline: variantInline, placeholder: variantPlaceholder, connecting: variantConnecting } } })>
  <row inline nowrap align="center" justify="center" hidden="true" style(icon, { box: shrink(1), paint: bg(empty-state).fg(tone), when: { empty: { box: display(none) }, forced-colors: { paint: bg(Canvas).fg(CanvasText) } }, compound: [ { tone: muted, paint: bg(empty-state).fg(empty-state.foreground) } ] })>
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
