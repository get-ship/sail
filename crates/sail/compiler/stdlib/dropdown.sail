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

  const host = style.position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).pad(1).fg(dropdown-foreground).bg(dropdown).borderWidth(1).border(border).radius(md).shadow(md).z(popover)
</scope>

<div popover use:anchor use:menu sx={host}><slot></slot></div>
