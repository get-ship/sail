<!--
  <spinner> — L1 language element. Style value language only.
  Rest paint = spinner.foreground when tone is omitted.
  Semantic paint = bound tone (fg-only). Authors omit tone for rest (never tone="default").
  Host subset: no muted product use historically.
-->

<scope>
  props: {
    size: 'md' | 'xs' | 'sm' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent'
    centered = false
  }

  const base = style({ box: display(inline-block).size(icon-size-md), paint: align(middle).fg(spinner.foreground) })
  const sizeMd = style({ box: size(icon-size-md) })
  const sizeXs = style({ box: size(icon-size-2xs) })
  const sizeSm = style({ box: size(icon-size-sm) })
  const sizeLg = style({ box: size(icon-size-xl) })
  const ring = style({ box: display(block).size(full), paint: borderWidth(control-stroke).border(solid).border(spinner.track).borderR(currentColor).radius(full).animate(spin) })
</scope>

<span style(base, { paint: fg(tone), match: { size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg } }, compound: [ { centered: true, box: display(block).my(8).mx(auto) } ] })>
  <span hidden="true" style(ring)></span>
</span>
