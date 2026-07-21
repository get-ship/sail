<!--
  <icon> — sizes/inks one glyph. use:icon fetches named/src glyphs.
  Host sx: align + ink + PE size md. CVA size maps box. SVG fill + spin fx
  are contracts (injected anatomy; no structure hook for the glyph).
-->

<scope>
  props: {
    name = ''
    src = ''
    label = ''
    // first key = default = PE md
    size: {
      md: 'size-icon-size-md',
      xs: 'size-icon-size-2xs',
      sm: 'size-icon-size-sm',
      lg: 'size-icon-size-xl',
      xl: 'size-icon-size-2xl',
    }
    spin = false
  }
</scope>

<span use:icon sx="inline-block align-middle text-current size-icon-size-md shrink-0">
  <slot></slot>
</span>
