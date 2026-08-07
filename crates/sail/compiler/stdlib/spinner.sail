<!--
  <spinner> — L1 language element. Style value language only:
  style.* chain + match + compound + scope Style consts.
-->

<scope>
  props: {
    size: 'md' | 'xs' | 'sm' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' = 'default'
    centered = false
  }

  const base = style({ box: display(inline-block).size(icon-size-md), paint: align(middle).fg(spinner-foreground) })
  const sizeMd = style({ box: size(icon-size-md) })
  const sizeXs = style({ box: size(icon-size-2xs) })
  const sizeSm = style({ box: size(icon-size-sm) })
  const sizeLg = style({ box: size(icon-size-xl) })
  const ring = style({ box: display(block).size(full), paint: borderWidth(control-stroke).border(solid).border(spinner-track).borderR(currentColor).radius(full).animate(spin) })
</scope>

<span style([
  base,
  match(size, {
    md: sizeMd,
    xs: sizeXs,
    sm: sizeSm,
    lg: sizeLg,
  }),
  match(tone, {
    default: style({ paint: fg(spinner-foreground) }),
    primary: style({ paint: fg(primary) }),
    info: style({ paint: fg(info) }),
    success: style({ paint: fg(success) }),
    warning: style({ paint: fg(warning) }),
    destructive: style({ paint: fg(destructive) }),
    accent: style({ paint: fg(accent) }),
  }),
  compound({ centered: true }, style({ box: display(block).my(8).mx(auto) })),
])>
  <span hidden="true" style(ring)></span>
</span>
