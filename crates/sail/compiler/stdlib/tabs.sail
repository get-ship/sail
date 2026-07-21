<!--
  <tabs> — tab strip + panels. use:tabs owns behavior + stamps orientation
  onto <tab-list>/<tab-panel>. Size channels via channels:. Layout owns vertical axis.
-->
<scope>
  props: {
    active = ''
    size: 'sm' | 'md' | 'lg' = 'md'
    tone = 'default'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }
  channels: {
    default: { 'tab-fs': 'text-sm', 'tab-py': 'space-2', 'tab-px': 'space-3' }
    size: {
      sm: { 'tab-fs': 'text-xs', 'tab-py': 'space-1_5', 'tab-px': 'space-2_5' }
      lg: { 'tab-fs': 'text-base', 'tab-py': 'space-2_5', 'tab-px': 'space-4' }
    }
  }
</scope>
<column gap="3" sx="min-w-0" use:tabs>
  <slot></slot>
</column>
