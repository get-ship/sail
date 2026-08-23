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
  // Shell paint inlined at the use site below (core Style syntax; no const refs).
  // Side sheets: radius/inset floor; edge margin + 100vh width live in DIALOG_MAP.

</scope>

<dialog style({ position({ scheme: fixed, inset: 0 }).pad(0).margin(auto).size({ w: 100%, max-w: overlay-max-w-md, min-w: overlay-max-w-sm, h: fit, max-h: overlay-vh-md }).display(flex).dir(column).overflow({ all: hidden }).border({ color: border, width: 1 }).radius(xl).bg(overlay).fg(foreground).elevation(xl), animate: { transition: play(surface, fast), enter: play(fade), exit: play(fade) }, match: {size: { sm: {size({ max-w: overlay-max-w-sm, min-w: 0 })}, md: {size({ max-w: overlay-max-w-md })}, lg: {size({ max-w: overlay-max-w-lg })}, full: {size({ max-w: 100%, max-h: 100%, min-w: 0, h: 100% })} }, side: {center: {margin(auto)}, start: {position({ inset: 0 }).size({ max-w: overlay-max-w-sm }).margin(0).radius(none)}, end: {position({ inset: 0 }).size({ max-w: overlay-max-w-sm }).margin(0).radius(none)}, top: {position({ inset: 0 }).size({ w: 100%, max-w: none }).margin(0).radius(none)}, bottom: {position({ inset: 0 }).size({ w: 100%, max-w: none }).margin(0).radius(none)}}}})>
  <slot></slot>
</dialog>
