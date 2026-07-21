<!--
  <tabs> — tab strip + panels. use:tabs owns behavior.

  Host: column+gap+min-w layout. Size channels (for <tab>), orientation row,
  sticky list, and pill/enclosed variant matrices are contracts (parent-keyed).
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
