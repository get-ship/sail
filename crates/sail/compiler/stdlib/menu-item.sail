<!--
  <menu-item> — item inside dropdown/context-menu. Style product host paint.
  Button L1 contract supplies inline-flex/align/gap chrome.

  Paint: fg / hover-bg from component.menu-item tokens (cssNames
  menu-item-foreground, menu-item-hover-bg).
-->

<scope>
  props: {
    value = ''
    disabled = false
  }

  // display(flex) not inline-flex: inline items sit on one line under a
  // max-content menu and the panel width becomes the SUM of labels.
  // padY(2)+padX(3)+gap(2_5) ≈ 32px hit, dense-IDE but breathable; icons align.
  // Button floor already supplies align-items:center.
  const base = style({ box: display(flex).gap(2_5).padY(2).padX(3).w(full), paint: borderWidth(0).fg(menu-item-foreground).bg(transparent).radius(sm).cursor(pointer), text: textAlign(start).fontFamily(body).fontSize(sm).leading(tight).whitespace(nowrap) })
</scope>

<button
  type="button" style(base, { when: { hover: { paint: bg(menu-item-hover-bg) }, focus: { paint: bg(menu-item-hover-bg).outline(solid).outline(2).outline(ring) }, disabled: { paint: opacity(50).cursor(not-allowed) } } })>
  <slot></slot>
</button>
