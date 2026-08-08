<!--
  <tab-panel> — orientation Style match (stamped by use:tabs).
-->
<scope>
  props: {
    name = ''
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }

  style base = { box: minW(0) }
  style horizontal = { box: padT(4).minW(0) }
  style vertical = { box: padT(0).padS(4).minW(0) }
</scope>
<div
  style(base, {
    match: { orientation: { horizontal: horizontal, vertical: vertical } },
  })
><slot></slot></div>
