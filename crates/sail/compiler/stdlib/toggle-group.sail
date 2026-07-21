<!--
  <toggle-group> — segmented toggle group. use:toggleGroup owns behavior.

  Host paint: inline row layout (gap 0 default). Residue: size channels,
  orientation column, outline join radii, ghost/segmented variant matrices.
-->

<scope>
  props: {
    type: 'single' | 'multiple' = 'single'
    value = ''
    variant: 'outline' | 'ghost' | 'segmented' = 'outline'
    size: 'sm' | 'md' | 'lg' = 'md'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }
</scope>

<row inline gap="0" use:toggleGroup>
  <slot></slot>
</row>
