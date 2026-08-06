<!--
  <progress> — determinate / indeterminate bar. Style product only.
  Vendor fill + indeterminate shimmer: PROGRESS_MAP.
-->

<scope>
  props: {
    value = ''
    max = 1
    size: 'sm' | 'md' | 'lg' = 'md'
  }

  const base = style({ box: display(inline-block).w(full).h(2).overflow(hidden), paint: appearance(none).align(middle).borderWidth(0).radius(full).bg(muted).fg(primary) })
  const sizeSm = style({ box: h(1_5) })
  const sizeMd = style({ box: h(2) })
  const sizeLg = style({ box: h(3) })
</scope>

<progress>
  style([
  base,
  match(size, {
    sm: sizeSm,
    md: sizeMd,
    lg: sizeLg,
  }),
])
</progress>
