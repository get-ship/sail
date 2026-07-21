<!--
  <toggle-group> — channels --tg-*; segmented CVA; stamps variant/orientation on cells.
-->
<scope>
  props: {
    type: 'single' | 'multiple' = 'single'
    value = ''
    variant: {
      outline: '',
      ghost: '',
      segmented: 'p-1 bg-muted border border-border rounded-md',
    }
    size: 'sm' | 'md' | 'lg' = 'md'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }
  channels: {
    default: { 'tg-fs': 'text-sm', 'tg-h': 'control-h-md', 'tg-px': 'space-3' }
    size: {
      sm: { 'tg-fs': 'text-xs', 'tg-h': 'control-h-sm', 'tg-px': 'space-2_5' }
      lg: { 'tg-fs': 'text-base', 'tg-h': 'control-h-lg', 'tg-px': 'space-4' }
    }
  }
</scope>
<row inline gap="0" use:toggleGroup>
  <slot></slot>
</row>
