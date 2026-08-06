<!--
  <tabs> — tab strip + panels. use:tabs owns behavior + stamps orientation
  onto <tab-list>/<tab-panel>. Size Style match lives on <tab>.
-->
<scope>
  props: {
    active = ''
    size: 'sm' | 'md' | 'lg' = 'md'
    tone = 'default'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }

  const host = style({ box: minW(0) })
</scope>
<column gap="3" use:tabs>
  style(host)

  <slot></slot>
</column>
