<!--
  <navbar> — application nav bar. Placements top/rail/bottom; sticky; bordered.
  Links use display(contents) so slot kids share the host flex axis.
  Link/label chrome is author style on projected nodes (current:/hover:).
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'rail' = 'top'
    sticky = false
    bordered = false
    offset: number = 0
  }

  style base = {
    paint: bg(navbar).fg(navbar.foreground),
    text: underline(none),
  }
  // Top + bottom share bar geometry; rail is a column panel.
  style placeBar = { box: minH(14).padY(2).padX(4) }
  style placeRail = { box: minH(0).padY(5).padX(4).minW(panel-rail-w).flexDir(column) }
  style brand = { text: fontWeight(semibold) }
  style links = { box: display(contents) }
  // Top/bottom: ms(auto) only. mt(auto) on a tall cross-axis drops actions to a
  // second visual row. Rail needs mt(auto) so actions pin to the column foot.
  style actions = {
    match: {
      placement: {
        rail: { box: ms(auto).mt(auto) },
        top: { box: ms(auto) },
        bottom: { box: ms(auto) },
      },
    },
  }
</scope>

<row
  nowrap
  align="center"
  gap="4"
  style(base, {
    box: w(full),
    match: {
      placement: {
        top: placeBar,
        bottom: placeBar,
        rail: placeRail,
      },
    },
    compound: [
      { sticky: true, box: position(sticky).top(bind(offset)), paint: z(sticky) },
      { bordered: true, paint: borderBWidth(1).border(navbar.border) },
      { placement: bottom, sticky: true, box: top(auto).bottom(0) },
      { bordered: true, placement: bottom, paint: borderBWidth(0).borderTWidth(1).border(navbar.border) },
      { bordered: true, placement: rail, paint: borderBWidth(0).borderEWidth(1).border(navbar.border) },
      {
        placement: rail,
        sticky: true,
        box: position(sticky).top(bind(offset)).bottom(0).self(start).overflowY(auto),
        paint: z(dropdown),
      },
    ],
  })
>
  <row inline nowrap align="center" gap="2" style(brand, { box: shrink(0) })>
    <slot name="brand"></slot>
  </row>
  <div style(links)>
    <slot></slot>
  </div>
  <row nowrap align="center" justify="center" style({ box: grow(1).minW(0) })>
    <slot name="center"></slot>
  </row>
  <row inline nowrap align="center" gap="2" style(actions, { box: shrink(0) })>
    <slot name="actions"></slot>
  </row>
</row>
