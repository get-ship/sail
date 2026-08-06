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
  const base = style({ box: position(fixed).inset(0).m(auto).pad(0).w(full).maxW(panel-max-w-md).minW(panel-max-w-sm).overflow(hidden), paint: borderWidth(1).border(border).radius(xl).bg(surface-1).fg(card-foreground).shadow(xl), text: fontFamily(body) })
  const open = style({ box: display(flex).flexDir(column).h(fit).maxH(panel-vh-md) })
  const sizeSm = style({ box: maxW(panel-max-w-sm).minW(0) })
  const sizeMd = style({ box: maxW(panel-max-w-md) })
  const sizeLg = style({ box: maxW(panel-max-w-lg) })
  const sizeFull = style({ box: maxW(full).maxH(full).minW(0).h(full) })
  // Side sheets: radius/inset floor; edge margin + 100vh width live in DIALOG_MAP.
  const sideStart = style({ box: inset(0).maxW(panel-max-w-sm).m(0), paint: radius(none) })
  const sideEnd = style({ box: inset(0).maxW(panel-max-w-sm).m(0), paint: radius(none) })
  const sideTop = style({ box: inset(0).w(full).maxW(none).m(0), paint: radius(none) })
  const sideBottom = style({ box: inset(0).w(full).maxW(none).m(0), paint: radius(none) })
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
    center: style({ box: m(auto) }),
    start: sideStart,
    end: sideEnd,
    top: sideTop,
    bottom: sideBottom,
  }),
])

  <slot></slot>
</dialog>
