<!--
  <navbar> — application nav bar. Placements top/rail/bottom; sticky; bordered.

  Paint/layout (Style product):
    layout keywords  — host row (nowrap, align center, gap 4); brand/actions
                       inline rows; links as contents so slot kids share the
                       host flex axis (row top / column rail); center row grows
    host sx={[…]}    — component.navbar tokens + match(placement) + compound
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

  const base = style.bg(navbar).fg(navbar-foreground).minH(14).padY(2).padX(4).underline(none)
  const placeTop = style.minH(14).padY(2).padX(4)
  const placeBottom = style.minH(14).padY(2).padX(4)
  const placeRail = style.minH(0).padY(5).padX(4).minW(panel-rail-w).flexDir(column)
  const brand = style.fontWeight(semibold)
  const links = style.display(contents)
  // Top/bottom: ms(auto) only. mt(auto) on a tall cross-axis drops actions to a
  // second visual row (looks like the nav "wrapped"). Rail needs mt(auto) so
  // actions pin to the foot of the column.
  const actions = match(placement, {
    rail: style.ms(auto).mt(auto),
    top: style.ms(auto),
    bottom: style.ms(auto),
  })
</scope>

<row
  nowrap
  align="center"
  gap="4"
  role="navigation"
  w="full"
  sx={[
    base,
    match(placement, {
      top: placeTop,
      bottom: placeBottom,
      rail: placeRail,
    }),
    compound({ sticky: true }, style.position(sticky).top(0).z(sticky)),
    compound({ bordered: true }, style.borderBWidth(1).border(border)),
    compound({ placement: bottom, sticky: true }, style.top(auto).bottom(0)),
    compound({ placement: bottom, bordered: true }, style.borderBWidth(0).borderTWidth(1).border(border)),
    compound({ placement: rail, bordered: true }, style.borderBWidth(0).borderEWidth(1).border(border)),
    compound({ placement: rail, sticky: true }, style.position(sticky).top(0).bottom(0).self(start).overflowY(auto).z(dropdown)),
    compound({ placement: rail, sticky: true, offset: '8' }, style.top(8)),
    compound({ placement: rail, sticky: true, offset: '10' }, style.top(10)),
    compound({ placement: rail, sticky: true, offset: '12' }, style.top(12)),
    compound({ placement: rail, sticky: true, offset: '14' }, style.top(14)),
    compound({ placement: rail, sticky: true, offset: '16' }, style.top(16)),
    compound({ placement: rail, sticky: true, offset: '20' }, style.top(20)),
    compound({ placement: rail, sticky: true, offset: '24' }, style.top(24)),
  ]}
>
  <row inline nowrap align="center" gap="2" shrink="0" sx={brand}>
    <slot name="brand"></slot>
  </row>
  <div sx={links}>
    <slot></slot>
  </div>
  <row
    nowrap
    align="center"
    justify="center"
    grow
    min-w="0"
  >
    <slot name="center"></slot>
  </row>
  <row
    inline
    nowrap
    align="center"
    gap="2"
    shrink="0"
    sx={actions}
  >
    <slot name="actions"></slot>
  </row>
</row>
