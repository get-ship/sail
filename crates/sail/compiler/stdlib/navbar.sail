<!--
  <navbar> — application nav bar. Placements top/rail/bottom; sticky; bordered.

  Top/bottom geometry: brand (start) · center (true center, absolute) · actions (end).
  Absolute center avoids flex asymmetry from unequal brand vs actions widths.

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
  style placeBar = { box: position(relative).minH(14).padY(2).padX(4) }
  style placeRail = { box: minH(0).padY(5).padX(4).minW(panel-rail-w).flexDir(column) }
  style brand = { box: shrink(0).minW(0), text: fontWeight(semibold) }
  // True geometric center: fill the bar, then justify-center the pills.
  // MUST use inset(0) (or w(full)+h(full)) — left(0).right(0).top(0).bottom(0)
  // emits s-right0/s-bottom0 atoms that are often ABSENT from the global
  // skins sheet, so right/bottom stay auto and the track shrink-wraps to
  // content width at left:0 (nav stuck under the brand). inset(0) is present
  // in element-skins and fills the relative bar.
  style centerTrack = {
    box: position(absolute).inset(0).pointerEvents(none).minW(0),
  }
  style centerInner = { box: pointerEvents(auto).minW(0).maxW(full) }
  style actions = { box: shrink(0).ms(auto).minW(0) }
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
  <row inline nowrap align="center" justify="start" gap="2" style(brand)>
    <slot name="brand"></slot>
  </row>
  <row nowrap align="center" justify="center" style(centerTrack)>
    <row nowrap align="center" justify="center" gap="2" style(centerInner)>
      <slot name="center"></slot>
      <slot></slot>
    </row>
  </row>
  <row inline nowrap align="center" justify="end" gap="2" style(actions)>
    <slot name="actions"></slot>
  </row>
</row>
