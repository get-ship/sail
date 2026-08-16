<!--
  <navbar> — application nav bar. Placements top/rail/bottom; sticky; bordered.

  Top/bottom geometry: brand (start) · center (true center, absolute) · actions (end).
  Absolute center avoids flex asymmetry from unequal brand vs actions widths.

  autohide: with sticky + use:chromeOffset, bar tucks on scroll-down and returns
  on scroll-up or pointer near the top edge (space reclaim without losing nav).
  chromeOffset stamps data-hidden; this skin turns off pointer-events so the
  reveal-edge can receive the hover. Do not use HTML hidden / display:none.

  floating: inset chrome — margin + radius + elevation, not edge-to-edge.

  solid-on-scroll: LANGUAGE GAP — Style cannot key on scroll position.
  chromeOffset measures/hides chrome but does not toggle solid paint.
  A new use:solidOnScroll would only flip a host attribute; skip JS.
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'rail' = 'top'
    sticky = false
    bordered = false
    autohide = false
    floating = false
    'solid-on-scroll' = false
    expanded = false
    offset: number = 0
  }

  // Width is a placement concern. w(full) on base made the rail the
  // viewport (PW: navbar 1440×2207, h1 at x=1440). Bar sets full;
  // rail sets fit-content. Do not put both on the host.
  style base = {position(relative).pad({ x: 4 }).bg(background).fg(foreground).type({ decoration: none }).group()}
  style placeBar = {position(relative).size({ w: full, min-h: 12 }).pad({ x: 4 }).dir(row)}
  style placeRail = {shrink(0).size({ w: fit, min-w: fit, min-h: fill }).pad({ y: 5, x: 4 }).dir(column)}
  // Expanded rail is a labeled menu (14rem). Collapsed stays icon-fit.
  style placeRailExp = {shrink(0).size({ w: panel-rail-w, min-w: panel-rail-w, min-h: fill }).pad({ y: 5, x: 3 }).dir(column)}
  style brand = {shrink(0).size({ min-w: 0 }).type({ weight: semibold })}
  // True geometric center: fill the bar, then justify-center the pills.
  // MUST use inset(0) (or w(full)+h(full)) — left(0).right(0).top(0).bottom(0)
  // emits s-right0/s-bottom0 atoms that are often ABSENT from the global
  // skins sheet, so right/bottom stay auto and the track shrink-wraps to
  // content width at left:0 (nav stuck under the brand). inset(0) is present
  // in element-skins and fills the relative bar.
  style centerTrack = {position({ scheme: absolute, inset: 0 }).size({ min-w: 0 }).pointerEvents(none)}
  // Rail is a stack: brand, links, actions. Absolute center is a top-bar trick.
  style centerTrackRail = {position(relative).pointerEvents(auto)}
  style centerInner = {pointerEvents(auto).size({ min-w: 0, max-w: full })}
  style centerInnerRail = {dir(column)}
  style actions = {shrink(0).margin({ s: auto }).size({ min-w: 0 })}
  style actionsRail = {margin({ t: auto, s: 0 }).dir(column)}
  // Instant tuck is state: use:chromeOffset stamps data-hidden. Pointer-events
  // none so the bar does not steal clicks; a fixed top edge still reveals.
  // translateY(-100%) is residual on the action — Style translateY is space
  // scale (translateY(-full/-100/-screen) do not compile).
  style tuck = {pointerEvents(none)}
</scope>

<row
  nowrap
  align="center"
  gap="4"
  label="Application"
  style(base, {match: { placement: { top: placeBar, bottom: placeBar, rail: placeRail }, 'data-hidden': { true: tuck } }, compound: [
      {sticky: true, position({ scheme: sticky, t: bind(offset), z: sticky })},
      {autohide: true, sticky: true, position({ scheme: fixed, l: 0, r: 0, t: 0, z: sticky }).size({ w: full }).transition(transform).duration(fast).ease(out)},
      {bordered: true, border({ color: border, edge: l }).border({ color: border, edge: r }).border({ color: border, edge: t }).border({ color: border, width: 1, edge: b })},
      {placement: bottom, sticky: true, position({ t: auto, b: 0 })},
      {bordered: true, placement: bottom, border({ color: border, edge: l }).border({ color: border, edge: r }).border({ color: border, width: 0, edge: b }).border({ color: border, width: 1, edge: t })},
      {bordered: true, placement: rail, border({ color: border, edge: s }).border({ color: border, edge: t }).border({ color: border, width: 0, edge: b }).border({ color: border, width: 1, edge: e })},
      {placement: rail, sticky: true, position({ scheme: sticky, t: bind(offset), b: 0, z: dropdown }).self(stretch).overflow({ y: auto })},
      {placement: rail, expanded: true, size({ w: panel-rail-w, min-w: panel-rail-w, min-h: fill }).pad({ y: 5, x: 3 })},
      {floating: true, margin(3).radius(lg).elevation(md)},
      {floating: true, autohide: true, sticky: true, position({ scheme: fixed, l: 3, r: 3, t: 3, z: sticky }).margin(0)},
    ]})
>
  <row inline nowrap align="center" justify="start" gap="2" style(brand)>
    <slot name="brand"></slot>
  </row>
  <row nowrap align="center" justify="center" style(centerTrack, {when: { group-placement-rail: centerTrackRail }})>
    <row nowrap align="center" justify="center" gap="2" style(centerInner, {when: { group-placement-rail: centerInnerRail }})>
      <slot name="center"></slot>
      <slot></slot>
    </row>
  </row>
  <row inline nowrap align="center" justify="end" gap="2" style(actions, {when: { group-placement-rail: actionsRail }})>
    <slot name="actions"></slot>
  </row>
</row>
