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
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'sm' | 'md' | 'lg' = 'md'
    name = ''
  }

  const base = style.appearance(none).w(full).h(5).bg(transparent).cursor(pointer).when({
    disabled: style.cursor(not-allowed).opacity(50),
  })
  const sizeSm = style.h(4)
  const sizeMd = style.h(5)
  const sizeLg = style.h(6)
</scope>

<input type="range" use:rangeFill sx={[
  base,
  match(size, {
    sm: sizeSm,
    md: sizeMd,
    lg: sizeLg,
  }),
]} />
