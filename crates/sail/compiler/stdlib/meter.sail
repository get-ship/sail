<!--
  <meter> — gauge (value-in-range). Style product only.
  WebKit fill bands + Firefox ::-moz-meter-bar: METER_MAP.
-->

<scope>
  props: {
    value = 0
    min = 0
    max = 1
    low = 0
    high = 1
    optimum = 1
    size: 'sm' | 'md' | 'lg' = 'md'
  }

  // Track rides component.meter tokens (global fallbacks in the pack).
  style base = {
    box: display(inline-block).w(full).h(2).overflow(hidden),
    paint: appearance(none).align(middle).borderWidth(0).radius(full).bg(meter),
  }
  style sizeSm = { box: h(1_5) }
  style sizeMd = { box: h(2) }
  style sizeLg = { box: h(3) }
</scope>

<meter style(base, {
  match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } },
})></meter>
