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

  const base = style.appearance(none).display(inline-block).align(middle).w(full).h(2).borderWidth(0).radius(full).overflow(hidden).bg(muted)
  const sizeSm = style.h(1_5)
  const sizeMd = style.h(2)
  const sizeLg = style.h(3)
</scope>

<meter sx={[
  base,
  match(size, {
    sm: sizeSm,
    md: sizeMd,
    lg: sizeLg,
  }),
]}></meter>
