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
  style base = {box: flow({ display: inline-block }).size({ w: full, h: 2 }).overflow({ all: hidden }), paint: appearance(none).align(middle).border({ width: 0 }).radius(full).fill({ color: muted }).ink({ color: primary })}
  style sizeSm = {box: size({ h: 1_5 })}
  style sizeMd = {box: size({ h: 2 })}
  style sizeLg = {box: size({ h: 3 })}
</scope>

<progress style(base, {
  match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } },
})></progress>
