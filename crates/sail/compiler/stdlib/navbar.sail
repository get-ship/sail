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

  style base = {bg(background).fg(foreground).type({ decoration: none })}
  style placeBar = {position(relative).size({ min-h: 12 }).pad({ x: 4 })}
  style placeRail = {size({ min-h: 0, min-w: panel-rail-w }).pad({ y: 5, x: 4 }).dir(column)}
  style brand = {shrink(0).size({ min-w: 0 }).type({ weight: semibold })}
  // True geometric center: fill the bar, then justify-center the pills.
  // MUST use inset(0) (or w(full)+h(full)) — left(0).right(0).top(0).bottom(0)
  // emits s-right0/s-bottom0 atoms that are often ABSENT from the global
  // skins sheet, so right/bottom stay auto and the track shrink-wraps to
  // content width at left:0 (nav stuck under the brand). inset(0) is present
  // in element-skins and fills the relative bar.
  style centerTrack = {position({ scheme: absolute, inset: 0 }).size({ min-w: 0 }).pointerEvents(none)}
  style centerInner = {pointerEvents(auto).size({ min-w: 0, max-w: full })}
  style actions = {shrink(0).margin({ s: auto }).size({ min-w: 0 })}
</scope>

<row
  nowrap
  align="center"
  gap="4"
  style(base, {size({ w: full }), match: { placement: { top: placeBar, bottom: placeBar, rail: placeRail } }, compound: [
      {sticky: true, position({ scheme: sticky, t: bind(offset), z: sticky })},
      {autohide: true, sticky: true, position({ scheme: fixed, l: 0, r: 0, t: 0, z: sticky }).size({ w: full }).motion({ property: transform, duration: fast, ease: out })},
      {bordered: true, border({ color: border, edge: l }).border({ color: border, edge: r }).border({ color: border, edge: t }).border({ color: border, width: 1, edge: b })},
      {placement: bottom, sticky: true, position({ t: auto, b: 0 })},
      {bordered: true, placement: bottom, border({ color: border, edge: l }).border({ color: border, edge: r }).border({ color: border, width: 0, edge: b }).border({ color: border, width: 1, edge: t })},
      {bordered: true, placement: rail, border({ color: border, edge: s }).border({ color: border, edge: t }).border({ color: border, width: 0, edge: b }).border({ color: border, width: 1, edge: e })},
      {placement: rail, sticky: true, position({ scheme: sticky, t: bind(offset), b: 0, z: dropdown }).self(start).overflow({ y: auto })},
    ]})
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
