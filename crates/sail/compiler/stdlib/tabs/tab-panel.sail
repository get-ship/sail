<!--
  <tab-panel> — orientation Style match (stamped by use:tabs).
-->
<scope>
  props: {
    name = ''
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }

  style base = {size({ min-w: 0 })}
  style horizontal = {pad({ t: 4 }).size({ w: 100%, min-w: 0 })}
  style vertical = {pad({ t: 0, s: 4 }).grow(1).size({ min-w: 0 })}
</scope>
<div
  style(base, { match: { orientation: { horizontal: horizontal, vertical: vertical } } })
><slot></slot></div>
