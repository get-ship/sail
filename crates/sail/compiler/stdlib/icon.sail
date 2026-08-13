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

  style base = {box: flow({ display: inline-block }).size(icon-size-md), paint: align(middle).ink({ color: currentColor }).glyphFill(true)}
  style sizeMd = {box: size(icon-size-md)}
  style sizeXs = {box: size(icon-size-2xs)}
  style sizeSm = {box: size(icon-size-sm)}
  style sizeLg = {box: size(icon-size-xl)}
  style sizeXl = {box: size(icon-size-2xl)}
</scope>
<span use:icon style(base, {box: flow({ shrink: 0 }), match: { size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg, xl: sizeXl } }})>
  <slot></slot>
</span>
