<!--
  <tab-panel> — orientation Style match (stamped by use:tabs).
-->
<scope>
  props: {
    name = ''
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }

  const base = style({ box: minW(0) })
  const horizontal = style({ box: padT(4).minW(0) })
  const vertical = style({ box: padT(0).padS(4).minW(0) })
</scope>
<div style(base, { match: { orientation: { horizontal: horizontal, vertical: vertical } } })><slot></slot></div>
