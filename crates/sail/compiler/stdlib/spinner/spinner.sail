<!--
  <spinner> — L1 language element. Style value language only.
  tone defaults to info — a bare <spinner> is a finished wait ring.
  Semantic paint = bound tone (fg-only).
  Host subset omits muted (no product muted rest path).
-->

<scope>
  props: {
    size: 'md' | 'xs' | 'sm' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' = 'info'
    centered = false
  }

  // Icon-size steps (10/14/16/20) were too tight to read as a size attr.
  // 16 / 24 / 32 / 48 is a visible glyph ladder; default md stays a mark.
  style base = {display(inline-block).size(8).align(middle).fg(muted-foreground)}
  style sizeMd = {size(8)}
  style sizeXs = {size(icon-size-md)}
  style sizeSm = {size(6)}
  style sizeLg = {size(12)}
  style ring = {display(block).size(full).border({ color: border-subtle, width: control-stroke, style: solid, edge: b }).border({ color: border-subtle, width: control-stroke, style: solid, edge: l }).border({ color: border-subtle, width: control-stroke, style: solid, edge: t }).border({ color: currentColor, width: control-stroke, style: solid, edge: r }).radius(full).animate(spin)}
</scope>

<span
  style(base, {fg(tone), match: { size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg } }, compound: [{centered: true, display(block).margin({ y: 8, x: auto })}]})
>
  <span hidden="true" style(ring)></span>
</span>
