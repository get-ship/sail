<!--
  <icon> — glyph box + size match. use:icon owns spin.
  Child svg pack lives in platform/icon.css.
-->
<scope>
  props: {
    name = ''
    src = ''
    label = ''
    size: 'md' | 'xs' | 'sm' | 'lg' | 'xl' = 'md'
    spin = false
  }

  style base = {display(inline-block).size(icon-size-md).align(middle).fg(currentColor)}
  style sizeMd = {size(icon-size-md)}
  style sizeXs = {size(icon-size-2xs)}
  style sizeSm = {size(icon-size-sm)}
  style sizeLg = {size(icon-size-xl)}
  style sizeXl = {size(icon-size-2xl)}
</scope>
<span use:icon style(base, {shrink(0), match: { size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg, xl: sizeXl } }})>
  <slot></slot>
</span>
