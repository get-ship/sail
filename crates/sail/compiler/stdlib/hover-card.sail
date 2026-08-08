<!--
  <hover-card> — L1 hover-preview overlay. Style value language only.
  Paint from component.hover-card tokens; w(max) hugs content (no rail minW).
-->

<scope>
  props: {
    placement = ''
  }

  style panel = { box: position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).w(max).maxW(panel-max-w-sm).pad(3), paint: fg(hover-card.foreground).bg(hover-card).borderWidth(1).border(hover-card.border).radius(md).shadow(md).z(popover), text: fontFamily(body).fontSize(sm) }
</scope>

<div popover use:anchor use:hoverSurface style(panel)><slot></slot></div>
