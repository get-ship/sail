<!--
  <toggle-group> — segmented toggle group. use:toggleGroup owns behavior.

  Host: inline row layout (gap 0 PE). Size channels, orientation column, outline
  join radii, and ghost/segmented child matrices are contracts (parent-keyed).
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
