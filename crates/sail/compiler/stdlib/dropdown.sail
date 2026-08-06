<!--
  <dropdown id="…" placement="bottom-start"> — L1 anchored menu (element mode).
  Native [popover] + use:anchor + use:menu. Compose <menu-item>s.

  Paint (Style product): bg/fg from component.dropdown tokens (cssNames dropdown,
  dropdown-foreground) + anchor geometry. Backdrop transparent is global
  :where([popover])::backdrop. min-width from role=menu contract (use:menu stamps role).
-->

<scope>
  props: {
    placement = ''
  }

  // w(max)=max-content hugs the longest item. No minW floor — 12rem panel-menu-min-w
  // made short path menus look empty. Product scoped sx does not reach top-layer
  // popovers; sizing lives here. pad(1_5) gives a light panel inset around items.
  const host = style({ box: position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).w(max).pad(1_5), paint: fg(dropdown-foreground).bg(dropdown).borderWidth(1).border(border).radius(md).shadow(md).z(popover) })
</scope>

<div popover use:anchor use:menu>
  style(host)
<slot></slot></div>
