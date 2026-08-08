<!--
  <dropdown id="…" placement="bottom-start"> — L1 anchored menu (element mode).
  Native [popover] + use:anchor + use:menu. Compose <menu-item>s.
  Paint from component.dropdown tokens + anchor geometry.
  Backdrop transparent is global :where([popover])::backdrop.
-->

<scope>
  props: {
    placement = ''
    onSelect: (detail: { value: string }) => void
  }

  // w(max)=max-content hugs the longest item (no minW floor). pad(1_5) insets items.
  style host = { box: position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).w(max).pad(1_5), paint: fg(dropdown.foreground).bg(dropdown).borderWidth(1).border(dropdown.border).radius(md).shadow(md).z(popover) }
</scope>

<div popover use:anchor use:menu style(host)><slot></slot></div>
