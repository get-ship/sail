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

  const base = style({ box: display(inline-block).size(icon-size-md), paint: align(middle).fg(currentColor).glyphFill(true) })
  const sizeMd = style({ box: size(icon-size-md) })
  const sizeXs = style({ box: size(icon-size-2xs) })
  const sizeSm = style({ box: size(icon-size-sm) })
  const sizeLg = style({ box: size(icon-size-xl) })
  const sizeXl = style({ box: size(icon-size-2xl) })
</scope>
<span use:icon style(base, { box: shrink(0), match: { size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg, xl: sizeXl } } })>
  <slot></slot>
</span>
