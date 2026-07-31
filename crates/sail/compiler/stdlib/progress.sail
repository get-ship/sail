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

  const base = style.appearance(none).display(inline-block).align(middle).w(full).h(2).borderWidth(0).radius(full).overflow(hidden).bg(muted).fg(primary)
  const sizeSm = style.h(1_5)
  const sizeMd = style.h(2)
  const sizeLg = style.h(3)
</scope>

<progress sx={[
  base,
  match(size, {
    sm: sizeSm,
    md: sizeMd,
    lg: sizeLg,
  }),
]}></progress>
