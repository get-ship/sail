<!--
  <tabs> — tab strip + panels. use:tabs owns behavior.

  Host paint: column+gap+min-w layout. Residue: size channels, orientation row,
  sticky list, pill/enclosed variant matrices (child-keyed).
-->

<scope>
  props: {
    active = ''
    variant: 'line' | 'pill' | 'enclosed' = 'line'
    size: 'sm' | 'md' | 'lg' = 'md'
    tone = 'default'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    sticky = false
  }
</scope>

<column gap="3" sx="min-w-0" use:tabs>
  <slot></slot>
</column>
