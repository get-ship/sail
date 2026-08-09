<!--
  <navbar> — application nav bar. Placements top/rail/bottom; sticky; bordered.
  Three-track layout: brand | center | actions — equal flex sides so center is
  geometrically centered (not biased by asymmetric action width).

  autohide: with sticky + use:chromeOffset, bar tucks on scroll-down and returns
  on scroll-up or pointer near the top edge (space reclaim without losing nav).
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'rail' = 'top'
    sticky = false
    bordered = false
    autohide = false
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
  // Equal side tracks → true center for the middle slot.
  style side = { box: basis(0).grow(1).minW(0) }
  style centerTrack = { box: shrink(0).minW(0) }
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
      // Fixed top so scroll can tuck the bar without sticky reflow fights.
      {
        autohide: true,
        sticky: true,
        box: position(fixed).left(0).right(0).top(0).w(full),
        paint: z(sticky).transition(transform).duration(fast).ease(out),
      },
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
  <row inline nowrap align="center" justify="start" gap="2" style(brand, side)>
    <slot name="brand"></slot>
  </row>
  <row nowrap align="center" justify="center" gap="2" style(centerTrack)>
    <slot name="center"></slot>
    <slot></slot>
  </row>
  <row inline nowrap align="center" justify="end" gap="2" style(side)>
    <slot name="actions"></slot>
  </row>
</row>
