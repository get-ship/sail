<!--
  <dialog> — native modal / sheet. Style product only.
  showModal() owns top-layer, ESC, focus-trap. size / backdrop / side are props.
  Structure (header/section/footer/::backdrop) + side docking: DIALOG_MAP.
-->

<scope>
  props: {
    size: 'sm' | 'md' | 'lg' | 'full' = 'md'
    backdrop: 'default' | 'blur' | 'solid' = 'default'
    side: 'center' | 'start' | 'end' | 'top' | 'bottom' = 'center'
  }

  // pad 0 — air lives on >header / >section / >footer (DIALOG_MAP). Flat
  // content without those gets a :has() floor in the map so demos never
  // collapse into a zero-pad pill. h(fit) keeps inset:0 from stretching the
  // shell to the full viewport (section flex would otherwise eat empty space).
  const base = style.position(fixed).inset(0).m(auto).pad(0).w(full).maxW(panel-max-w-md).minW(panel-max-w-sm).overflow(hidden).borderWidth(1).border(border).radius(xl).bg(surface-1).fg(card-foreground).shadow(xl).fontFamily(body)
  const open = style.display(flex).flexDir(column).h(fit).maxH(panel-vh-md)
  const sizeSm = style.maxW(panel-max-w-sm).minW(0)
  const sizeMd = style.maxW(panel-max-w-md)
  const sizeLg = style.maxW(panel-max-w-lg)
  const sizeFull = style.maxW(full).maxH(full).minW(0).h(full)
  // Side sheets: radius/inset floor; edge margin + 100vh width live in DIALOG_MAP.
  const sideStart = style.inset(0).maxW(panel-max-w-sm).radius(none).m(0)
  const sideEnd = style.inset(0).maxW(panel-max-w-sm).radius(none).m(0)
  const sideTop = style.inset(0).w(full).maxW(none).radius(none).m(0)
  const sideBottom = style.inset(0).w(full).maxW(none).radius(none).m(0)
</scope>

<dialog>
  style([
  base,
  when({ open: open }),
  match(size, {
    sm: sizeSm,
    md: sizeMd,
    lg: sizeLg,
    full: sizeFull,
  }),
  match(side, {
    center: style.m(auto),
    start: sideStart,
    end: sideEnd,
    top: sideTop,
    bottom: sideBottom,
  }),
])

  <slot></slot>
</dialog>
