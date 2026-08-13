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

  // Track rides muted.
  style base = {box: flow({ display: inline-block }).size({ w: full, h: 2 }).overflow({ all: hidden }), paint: appearance(none).align(middle).border({ width: 0 }).radius(full).fill({ color: muted })}
  style sizeSm = {box: size({ h: 1_5 })}
  style sizeMd = {box: size({ h: 2 })}
  style sizeLg = {box: size({ h: 3 })}
</scope>

<meter style(base, {
  match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } },
})></meter>
