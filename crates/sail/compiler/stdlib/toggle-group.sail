<!--
  <toggle-group> — segmented CVA; structural orientation stamp on cells only.
  Size/variant paint is authored on each toggle.
-->
<scope>
  props: {
    type: 'single' | 'multiple' = 'single'
    value = ''
    variant: {
      outline: '',
      ghost: '',
      segmented: 'p: 1 bg: muted border border: border rounded: md',
    }
    size: 'sm' | 'md' | 'lg' = 'md'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }
</scope>
<row inline gap="0" use:toggleGroup>
  <slot></slot>
</row>
