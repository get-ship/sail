<!--
  <icon> — glyph-fill child pack for svg box; use:icon owns spin.
-->
<scope>
  props: {
    name = ''
    src = ''
    label = ''
    size: {
      md: 'size: icon-size-md',
      xs: 'size: icon-size-2xs',
      sm: 'size: icon-size-sm',
      lg: 'size: icon-size-xl',
      xl: 'size: icon-size-2xl',
    }
    spin = false
  }
</scope>
<span use:icon sx="inline-block align: middle color: currentColor size: icon-size-md shrink: 0 glyph-fill">
  <slot></slot>
</span>
