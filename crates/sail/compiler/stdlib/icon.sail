<!--
  <icon> — glyph box + size match. use:icon owns spin. glyphFill packs child svg.
-->
<scope>
  props: {
    name = ''
    src = ''
    label = ''
    size: 'md' | 'xs' | 'sm' | 'lg' | 'xl' = 'md'
    spin = false
  }

  const base = style.display(inline-block).align(middle).fg(currentColor).size(icon-size-md).glyphFill(true)
  const sizeMd = style.size(icon-size-md)
  const sizeXs = style.size(icon-size-2xs)
  const sizeSm = style.size(icon-size-sm)
  const sizeLg = style.size(icon-size-xl)
  const sizeXl = style.size(icon-size-2xl)
</scope>
<span use:icon shrink="0" sx={[
  base,
  match(size, {
    md: sizeMd,
    xs: sizeXs,
    sm: sizeSm,
    lg: sizeLg,
    xl: sizeXl,
  }),
]}>
  <slot></slot>
</span>
