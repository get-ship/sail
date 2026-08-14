<!--
  <dropdown id="…" placement="bottom-start"> — L1 anchored menu (element mode).
  Native [popover] + use:anchor + use:menu. Compose <menu-item>s.
  Paint from popover / foreground / border globals + anchor geometry.
  Backdrop transparent is global :where([popover])::backdrop.
-->

<scope>
  props: {
    placement = ''
    onSelect: (detail: { value: string }) => void
  }

  // w(max)=max-content hugs the longest item (no minW floor). pad(1_5) insets items.
  style host = {position({ scheme: fixed, inset: auto, l: bind(anchorLeft), t: bind(anchorTop), z: popover }).pad(1_5).margin(0).size({ w: max }).fg(foreground).bg(background).border({ color: border, width: 1 }).radius(md).elevation(md)}
</scope>

<div popover use:anchor use:menu style(host)><slot></slot></div>
