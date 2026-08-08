<!--
  <navbar> — application nav bar. Placements top/rail/bottom; sticky; bordered.

  Paint/layout (Style product):
    layout keywords  — host row (nowrap, align center, gap 4); brand/actions
                       inline rows; links as contents so slot kids share the
                       host flex axis (row top / column rail); center row grows
    host sx={[…]}    — component.navbar tokens + style({ match: { placement: {  } } }) + compound
                       sticky/bordered/rail geometry (no CVA string maps)
    child sx         — center grow-1; actions ms-auto mt-auto
  Slotted link/label chrome is author sx on the projected nodes (current:/hover:).
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'rail' = 'top'
    sticky = false
    bordered = false
    offset = ''
  }

  const base = style({ box: minH(14).padY(2).padX(4), paint: bg(navbar).fg(navbar-foreground), text: underline(none) })
  const placeTop = style({ box: minH(14).padY(2).padX(4) })
  const placeBottom = style({ box: minH(14).padY(2).padX(4) })
  const placeRail = style({ box: minH(0).padY(5).padX(4).minW(panel-rail-w).flexDir(column) })
  const brand = style({ text: fontWeight(semibold) })
  const links = style({ box: display(contents) })
  // Top/bottom: ms(auto) only. mt(auto) on a tall cross-axis drops actions to a
  // second visual row (looks like the nav "wrapped"). Rail needs mt(auto) so
  // actions pin to the foot of the column.
  const actions = style({ match: { placement: { rail: { box: ms(auto).mt(auto) }, top: { box: ms(auto) }, bottom: { box: ms(auto) } } } })
</scope>

<row nowrap align="center" gap="4" style(base, { box: w(full), match: { placement: { top: placeTop, bottom: placeBottom, rail: placeRail } }, compound: [ { sticky: true, box: position(sticky).top(0), paint: z(sticky) }, { bordered: true, paint: borderBWidth(1).border(navbar-border) }, { placement: bottom, sticky: true, box: top(auto).bottom(0) }, { bordered: true, placement: bottom, paint: borderBWidth(0).borderTWidth(1).border(navbar-border) }, { bordered: true, placement: rail, paint: borderBWidth(0).borderEWidth(1).border(navbar-border) }, { placement: rail, sticky: true, box: position(sticky).top(0).bottom(0).self(start).overflowY(auto), paint: z(dropdown) }, { offset: 8, placement: rail, sticky: true, box: top(8) }, { offset: 10, placement: rail, sticky: true, box: top(10) }, { offset: 12, placement: rail, sticky: true, box: top(12) }, { offset: 14, placement: rail, sticky: true, box: top(14) }, { offset: 16, placement: rail, sticky: true, box: top(16) }, { offset: 20, placement: rail, sticky: true, box: top(20) }, { offset: 24, placement: rail, sticky: true, box: top(24) } ] })>
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
