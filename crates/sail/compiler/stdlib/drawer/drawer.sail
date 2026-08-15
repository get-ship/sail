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

  // Non-modal: no top layer, so z(dialog). inset:0 + opposite-edge margin auto
  // docks the used box (max-w / max-h) without min() platform geometry.
  // size sets both max-w (start/end) and max-h (top/bottom). Side sheets
  // cancel max-h so they stay viewport-tall.
  style host = {position({ scheme: fixed, inset: 0, z: dialog }).pad({ y: 5, x: 6 }).margin(0).size({ w: full, h: full }).overflow({ y: auto, overscroll: contain }).border({ color: border, width: 1 }).radius(none).bg(card).fg(card-foreground).elevation(xl).type({ family: body }), animate: { transition: play(surface, fast) }}
  style shown = {display(flex).dir(column)}
  style sizeSm = {size({ max-w: panel-max-w-sm, max-h: panel-vh-sm })}
  style sizeMd = {size({ max-w: panel-max-w-md, max-h: panel-vh-md })}
  style sizeLg = {size({ max-w: panel-max-w-lg, max-h: panel-vh-lg })}
  style sideStart = {size({ w: full, h: full, max-h: none }).margin({ e: auto }).border({ width: 0, edge: s }).touch(pan-y)}
  style sideEnd = {size({ w: full, h: full, max-h: none }).margin({ s: auto }).border({ width: 0, edge: e }).touch(pan-y)}
  style sideTop = {size({ w: full, max-w: none, h: fit }).margin({ b: auto }).border({ width: 0, edge: t }).touch(pan-x)}
  style sideBottom = {size({ w: full, max-w: none, h: fit }).margin({ t: auto }).border({ width: 0, edge: b }).touch(pan-x)}
</scope>

<dialog
  use:drawerSwipe
  style(host, {when: { open: shown }, match: {size: { sm: sizeSm, md: sizeMd, lg: sizeLg }, side: {start: sideStart, end: sideEnd, top: sideTop, bottom: sideBottom}}})
>
  <slot></slot>
</dialog>
