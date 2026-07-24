<!--
  <tabs> — tab strip + panels. use:tabs owns behavior + stamps orientation
  onto <tab-list>/<tab-panel>. Size CVA lives on <tab>.
-->
<scope>
  props: {
    active = ''
    size: 'sm' | 'md' | 'lg' = 'md'
    tone = 'default'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }
</scope>
<column gap="3" sx="min-w: 0" use:tabs>
  <slot></slot>
</column>
