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

  style base = {display(inline-block).size(icon-size-md).align(middle).fg(muted-foreground)}
  style sizeMd = {size(icon-size-md)}
  style sizeXs = {size(icon-size-2xs)}
  style sizeSm = {size(icon-size-sm)}
  style sizeLg = {size(icon-size-xl)}
  style ring = {display(block).size(full).border({ color: border-subtle, width: control-stroke, style: solid, edge: b }).border({ color: border-subtle, width: control-stroke, style: solid, edge: l }).border({ color: border-subtle, width: control-stroke, style: solid, edge: t }).border({ color: currentColor, width: control-stroke, style: solid, edge: r }).radius(full).animate(spin)}
</scope>

<span
  style(base, {fg(tone), match: { size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg } }, compound: [{centered: true, display(block).margin({ y: 8, x: auto })}]})
>
  <span hidden="true" style(ring)></span>
</span>
