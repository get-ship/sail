<!--
  <dialog> — native modal / sheet. Style product only.
  showModal() owns top-layer, ESC, focus-trap. size / backdrop / side / dismissible
  are props. dismissible=false → bindOpen opts (no ESC/backdrop dismiss).
  Structure (header/section/footer/::backdrop) + side docking: DIALOG_MAP.
-->

<scope>
  props: {
    size: 'sm' | 'md' | 'lg' | 'full' = 'md'
    backdrop: 'default' | 'blur' | 'solid' = 'default'
    side: 'center' | 'start' | 'end' | 'top' | 'bottom' = 'center'
    /** ESC + backdrop can close. Destructive confirms set false so the user must pick an action. */
    dismissible = true
    onClose: () => void
    onDismiss: () => void
  }

  // pad 0 — air lives on >header / >section / >footer (DIALOG_MAP). Flat content
  // without those gets a :has() floor in the map. h(fit) keeps inset:0 from
  // stretching the shell to the full viewport.
  style base = {box: place({ position: fixed, inset: 0 }).space({ m: auto, pad: 0 }).size({ w: full, max-w: panel-max-w-md, min-w: panel-max-w-sm }).overflow({ all: hidden }), paint: border({ color: border, width: 1 }).radius(xl).fill({ color: card }).ink({ color: card-foreground }).elevation(xl), text: type({ family: body })}
  style open = {box: flow({ display: flex, dir: column }).size({ h: fit, max-h: panel-vh-md })}
  style sizeSm = {box: size({ max-w: panel-max-w-sm, min-w: 0 })}
  style sizeMd = {box: size({ max-w: panel-max-w-md })}
  style sizeLg = {box: size({ max-w: panel-max-w-lg })}
  style sizeFull = {box: size({ max-w: full, max-h: full, min-w: 0, h: full })}
  // Side sheets: radius/inset floor; edge margin + 100vh width live in DIALOG_MAP.
  style sideEdge = {box: place({ inset: 0 }).size({ max-w: panel-max-w-sm }).space({ m: 0 }), paint: radius(none)}
  style sideBand = {box: place({ inset: 0 }).size({ w: full, max-w: none }).space({ m: 0 }), paint: radius(none)}
</scope>

<dialog style(base, {when: { open: open }, match: {size: { sm: sizeSm, md: sizeMd, lg: sizeLg, full: sizeFull }, side: {center: {box: space({ m: auto })}, start: sideEdge, end: sideEdge, top: sideBand, bottom: sideBand}}})>
  <slot></slot>
</dialog>
