<!--
  <spinner> — L1 language element. Style value language only.
  Rest paint = muted-foreground when tone is omitted.
  Semantic paint = bound tone (fg-only).
  Host subset omits muted (no product muted rest path).
-->

<scope>
  props: {
    size: 'md' | 'xs' | 'sm' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent'
    centered = false
  }

  style base = {box: flow({ display: inline-block }).size(icon-size-md), paint: align(middle).ink({ color: muted-foreground })}
  style sizeMd = {box: size(icon-size-md)}
  style sizeXs = {box: size(icon-size-2xs)}
  style sizeSm = {box: size(icon-size-sm)}
  style sizeLg = {box: size(icon-size-xl)}
  style ring = {box: flow({ display: block }).size(full), paint: border({ color: border-subtle, width: control-stroke, style: solid, edge: b }).border({ color: border-subtle, width: control-stroke, style: solid, edge: l }).border({ color: border-subtle, width: control-stroke, style: solid, edge: t }).border({ color: currentColor, width: control-stroke, style: solid, edge: r }).radius(full).motion({animate: spin})}
</scope>

<span
  style(base, {paint: ink({ color: tone }), match: { size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg } }, compound: [{centered: true, box: flow({ display: block }).space({ my: 8, mx: auto })}]})
>
  <span hidden="true" style(ring)></span>
</span>
