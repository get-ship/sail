<!--
  <drawer> — edge-anchored non-modal sheet. Style product only.
  Native <dialog> without showModal — invoker command="show" commandfor.
  side start|end|top|bottom (default end). size sm|md|lg. dismissible default true.
  Light-dom children: <header> / <section> / <footer> (same as dialog, not slots).
  use:drawerSwipe — swipe toward the edge closes; Escape + light-dismiss too.
  Edge geometry is v3 Style (margin auto + max-w/max-h). No platform CSS.
  Descendant header/section/footer chrome is a LANGUAGE GAP (DIALOG_MAP).
-->

<scope>
  props: {
    size: 'sm' | 'md' | 'lg' = 'md'
    side: 'start' | 'end' | 'top' | 'bottom' = 'end'
    /** Escape, outside pointer, and swipe-toward-edge can close. */
    dismissible = true
    onClose: () => void
    onDismiss: () => void
  }

  // Shell only. Edge dock + header/section/footer live in platform/drawer.css
  // (same native-<dialog> adaptor as dialog). inset:0 + w/h full made a
  // full-viewport card — that is not a drawer.
  style host = {position({ scheme: fixed, z: dialog }).pad(0).margin(0).overflow({ y: auto, overscroll: contain }).border({ color: border, width: 1 }).radius(none).bg(overlay).fg(foreground).elevation(xl).type({ family: body }), animate: { transition: play(surface, fast) }}
  style shown = {display(flex).dir(column)}
  style sizeSm = {size({ max-w: panel-max-w-sm, max-h: panel-vh-sm })}
  style sizeMd = {size({ max-w: panel-max-w-md, max-h: panel-vh-md })}
  style sizeLg = {size({ max-w: panel-max-w-lg, max-h: panel-vh-lg })}
  style sideStart = {touch(pan-y)}
  style sideEnd = {touch(pan-y)}
  style sideTop = {touch(pan-x)}
  style sideBottom = {touch(pan-x)}
</scope>

<dialog
  use:drawerSwipe
  style(host, {when: { open: shown }, match: {size: { sm: sizeSm, md: sizeMd, lg: sizeLg }, side: {start: sideStart, end: sideEnd, top: sideTop, bottom: sideBottom}}})
>
  <slot></slot>
</dialog>
