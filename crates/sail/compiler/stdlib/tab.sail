<!--
  <tab> — paint + variant/size Style match for pill/enclosed and pad scale.
  Paint: idle/selected from component.tab tokens
         (cssNames tab.foreground, tab.selected-fg, tab.selected-bg).
-->

<scope>
  props: {
    name = ''
    disabled = false
    variant: 'line' | 'pill' | 'enclosed' = 'line'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  style base = { box: padY(2).padX(3), paint: fg(tab.foreground).bg(transparent).cursor(pointer).select(none).borderWidth(0).borderBWidth(2).borderB(transparent).radiusT(sm).tapHighlight(none).transition(colors).duration(fast).ease(out), text: fontFamily(body).fontWeight(medium).leading(none).whitespace(nowrap).fontSize(sm) }
  style sizeMd = { box: padY(2).padX(3), text: fontSize(sm) }
  style sizeSm = { box: padY(1_5).padX(2_5), text: fontSize(xs) }
  style sizeLg = { box: padY(2_5).padX(4), text: fontSize(base) }
  style pill = { paint: borderBWidth(0).radius(sm) }
  style enclosed = { paint: borderWidth(1).border(transparent).borderBWidth(0).radiusT(md) }
</scope>

<button type="button" style(base, { when: { hover: { paint: fg(tab.selected-fg) }, selected: { paint: fg(tab.selected-fg).borderB(primary), text: fontWeight(semibold) }, focus: { paint: outline(solid).outline(ring-width).outline(ring).outlineOffset(ring-inset) }, disabled: { paint: opacity(50).cursor(not-allowed) } }, match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, variant: { line: { paint: borderB(transparent) }, pill: pill, enclosed: enclosed } }, compound: [ { variant: pill, paint: borderBWidth(0).radius(sm) }, { variant: enclosed, paint: borderWidth(1).border(transparent).borderBWidth(0).radiusT(md) }, { variant: pill, when: { selected: { paint: fg(tab.selected-fg).bg(tab.selected-bg).shadow(xs).borderB(transparent) } } }, { variant: enclosed, when: { selected: { paint: fg(tab.selected-fg).border(tab.border).bg(tab.selected-bg).translateY(px) } } } ] })>
  <slot></slot>
</button>
