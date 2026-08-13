<!--
  <tabs> — tab strip + panels. use:tabs owns behavior + stamps orientation
  onto <tab-list>/<tab-panel>. Size Style match lives on <tab> (author there).
-->
<scope>
  props: {
    active = ''
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    onTabChange: (detail: { name: string }) => void
  }

  style host = {box: size({ min-w: 0 })}
</scope>
<column gap="3" use:tabs style(host)>
  <slot></slot>
</column>
