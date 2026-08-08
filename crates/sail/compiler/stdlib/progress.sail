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

  // Track/fill ride component.progress tokens (global fallbacks in the pack).
  style base = {
    box: display(inline-block).w(full).h(2).overflow(hidden),
    paint: appearance(none).align(middle).borderWidth(0).radius(full).bg(progress).fg(progress.fill),
  }
  style sizeSm = { box: h(1_5) }
  style sizeMd = { box: h(2) }
  style sizeLg = { box: h(3) }
</scope>

<progress style(base, {
  match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } },
})></progress>
