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

  const base = style.w(full).pad(2).padX(2_5).borderWidth(0).textAlign(start).fontFamily(body).fontSize(sm).leading(tight).fg(menu-item-foreground).bg(transparent).radius(sm).cursor(pointer).whitespace(nowrap)
</scope>

<button
  type="button"
  sx={[
    base,
    when({
      hover: style.bg(menu-item-hover-bg),
      focus: style.bg(menu-item-hover-bg).outline(solid).outline(2).outline(ring),
      disabled: style.opacity(50).cursor(not-allowed),
    }),
  ]}
>
  <slot></slot>
</button>
