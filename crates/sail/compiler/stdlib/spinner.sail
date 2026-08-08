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

  style base = { box: display(inline-block).size(icon-size-md), paint: align(middle).fg(spinner.foreground) }
  style sizeMd = { box: size(icon-size-md) }
  style sizeXs = { box: size(icon-size-2xs) }
  style sizeSm = { box: size(icon-size-sm) }
  style sizeLg = { box: size(icon-size-xl) }
  style ring = { box: display(block).size(full), paint: borderWidth(control-stroke).border(solid).border(spinner.track).borderR(currentColor).radius(full).animate(spin) }
</scope>

<span style(base, { paint: fg(tone), match: { size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg } }, compound: [ { centered: true, box: display(block).my(8).mx(auto) } ] })>
  <span hidden="true" style(ring)></span>
</span>
