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

  style base = {paint: fill({ color: background }).ink({ color: foreground }), text: type({ decoration: none })}
  style placeBar = {box: place({ position: relative }).size({ min-h: 12 }).space({ pad-x: 4 })}
  style placeRail = {box: size({ min-h: 0, min-w: panel-rail-w }).space({ pad-y: 5, pad-x: 4 }).flow({ dir: column })}
  style brand = {box: flow({ shrink: 0 }).size({ min-w: 0 }), text: type({ weight: semibold })}
  // True geometric center: fill the bar, then justify-center the pills.
  // MUST use inset(0) (or w(full)+h(full)) — left(0).right(0).top(0).bottom(0)
  // emits s-right0/s-bottom0 atoms that are often ABSENT from the global
  // skins sheet, so right/bottom stay auto and the track shrink-wraps to
  // content width at left:0 (nav stuck under the brand). inset(0) is present
  // in element-skins and fills the relative bar.
  style centerTrack = {box: place({ position: absolute, inset: 0 }).size({ min-w: 0 }).pointerEvents(none)}
  style centerInner = {box: pointerEvents(auto).size({ min-w: 0, max-w: full })}
  style actions = {box: flow({ shrink: 0 }).space({ ms: auto }).size({ min-w: 0 })}
</scope>

<row
  nowrap
  align="center"
  gap="4"
  style(base, {box: size({ w: full }), match: {
      placement: {
        top: placeBar,
        bottom: placeBar,
        rail: placeRail,
      },
    }, compound: [
      {sticky: true, box: place({ position: sticky, top: bind(offset), z: sticky })},
      {autohide: true, sticky: true, box: place({ position: fixed, left: 0, right: 0, top: 0, z: sticky }).size({ w: full }), paint: motion({ property: transform, duration: fast, ease: out })},
      {bordered: true, paint: border({ color: border, edge: l }).border({ color: border, edge: r }).border({ color: border, edge: t }).border({ color: border, width: 1, edge: b })},
      {placement: bottom, sticky: true, box: place({ top: auto, bottom: 0 })},
      {bordered: true, placement: bottom, paint: border({ color: border, edge: l }).border({ color: border, edge: r }).border({ color: border, width: 0, edge: b }).border({ color: border, width: 1, edge: t })},
      {bordered: true, placement: rail, paint: border({ color: border, edge: s }).border({ color: border, edge: t }).border({ color: border, width: 0, edge: b }).border({ color: border, width: 1, edge: e })},
      {placement: rail, sticky: true, box: place({ position: sticky, top: bind(offset), bottom: 0, z: dropdown }).flow({ self: start }).overflow({ y: auto })},
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
