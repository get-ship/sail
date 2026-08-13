<!--
  <tab-panel> — orientation Style match (stamped by use:tabs).
-->
<scope>
  props: {
    name = ''
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }

  style base = {box: size({ min-w: 0 })}
  style horizontal = {box: space({ pad-t: 4 }).size({ min-w: 0 })}
  style vertical = {box: space({ pad-t: 0, pad-s: 4 }).size({ min-w: 0 })}
</scope>
<div
  style(base, {
    match: { orientation: { horizontal: horizontal, vertical: vertical } },
  })
><slot></slot></div>
