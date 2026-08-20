<!--
  <icon> — glyph box + size match. Pure structure: glyphs are compiler
  pack-inline, aria is anatomy (role=img when labeled, aria-hidden when
  decorative). No controller, no signals, no runtime loader.

  NOTE: `spin` was removed — conditional motion arms (match/if) hoist
  unconditionally in the compiler today, so any spin wiring spins every
  icon. Filed: prop-gated motion arms (motion_emit). Restore spin when
  that lands.
-->
<scope>
  props: {
    name = ''
    label = ''
    size: 'md' | 'xs' | 'sm' | 'lg' | 'xl' = 'md'
  }

  style base = {display(inline-block).size(icon-size-md).align(middle).fg(currentColor)}
  style sizeMd = {size(icon-size-md)}
  style sizeXs = {size(icon-size-2xs)}
  style sizeSm = {size(icon-size-sm)}
  style sizeLg = {size(icon-size-xl)}
  style sizeXl = {size(icon-size-2xl)}
</scope>
<span style(base, {shrink(0), match: { size: { md: sizeMd, xs: sizeXs, sm: sizeSm, lg: sizeLg, xl: sizeXl } }})>
  <slot></slot>
</span>