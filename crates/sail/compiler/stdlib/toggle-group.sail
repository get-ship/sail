<!--
  <toggle-group> — segmented Style match; structural orientation stamp on cells only.
  Size/variant paint is authored on each toggle.
  Paint: segmented rail from component.toggle-group tokens
         (cssNames toggle-group, toggle-group-border).
-->

<scope>
  props: {
    type: 'single' | 'multiple' = 'single'
    value = ''
    variant: 'outline' | 'ghost' | 'segmented' = 'outline'
    size: 'sm' | 'md' | 'lg' = 'md'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }

  const segmented = style.pad(1).bg(toggle-group).borderWidth(1).border(toggle-group-border).radius(md)
</scope>

<row inline gap="0" use:toggleGroup sx={[
  match(variant, {
    outline: style.bg(transparent),
    ghost: style.bg(transparent),
    segmented: segmented,
  }),
]}>
  <slot></slot>
</row>
