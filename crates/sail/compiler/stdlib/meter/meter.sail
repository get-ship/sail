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
  style base = {display(inline-block).size({ w: full, h: 2 }).overflow({ all: hidden }).appearance(none).align(middle).border({ width: 0 }).radius(full).bg(muted)}
  style sizeSm = {size({ h: 1_5 })}
  style sizeMd = {size({ h: 2 })}
  style sizeLg = {size({ h: 3 })}
</scope>

<meter style(base, { match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } } })></meter>
