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

  // Track/fill ride muted / primary.
  style base = {display(inline-block).size({ w: full, h: 2 }).overflow({ all: hidden }).appearance(none).align(middle).border({ width: 0 }).radius(full).bg(muted).fg(primary)}
  style sizeSm = {size({ h: 1_5 })}
  style sizeMd = {size({ h: 2 })}
  style sizeLg = {size({ h: 3 })}
</scope>

<progress style(base, { match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } } })></progress>
