<!--
  <slider> — native range alias. use:rangeFill → --rng-pct (WebKit fill).
  Track/thumb vendor pseudos: RANGE_MAP.
-->

<scope>
  props: {
    value = 0
    min = 0
    max = 100
    step = 1
    disabled = false
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    size: 'sm' | 'md' | 'lg' = 'md'
    name = ''
  }

  style base = {
    box: w(full).h(5),
    paint: appearance(none).bg(transparent).cursor(pointer),
    when: { disabled: { paint: cursor(not-allowed).opacity(50) } },
  }
  style sizeSm = { box: h(4) }
  style sizeMd = { box: h(5) }
  style sizeLg = { box: h(6) }
</scope>

<input
  type="range"
  use:rangeFill
  style(base, {
    match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } },
  })
>
