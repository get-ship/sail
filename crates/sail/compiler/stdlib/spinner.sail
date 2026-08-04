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

  const base = style.display(inline-block).align(middle).size(icon-size-md).fg(spinner-foreground)
  const sizeMd = style.size(icon-size-md)
  const sizeXs = style.size(icon-size-2xs)
  const sizeSm = style.size(icon-size-sm)
  const sizeLg = style.size(icon-size-xl)
  const ring = style.display(block).size(full).borderWidth(control-stroke).border(solid).border(spinner-track).borderR(currentColor).radius(full).animate(spin)
</scope>

<span sx={[
  base,
  match(size, {
    md: sizeMd,
    xs: sizeXs,
    sm: sizeSm,
    lg: sizeLg,
  }),
  match(tone, {
    default: style.fg(spinner-foreground),
    primary: style.fg(primary),
    info: style.fg(info),
    success: style.fg(success),
    warning: style.fg(warning),
    destructive: style.fg(destructive),
    accent: style.fg(accent),
  }),
  compound({ centered: true }, style.display(block).my(8).mx(auto)),
]}>
  <span hidden="true" sx={ring}></span>
</span>
