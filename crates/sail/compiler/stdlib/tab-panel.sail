<!--
  <tab-panel> — orientation CVA (stamped by use:tabs).
-->
<scope>
  props: {
    name = ''
    orientation: {
      horizontal: 'pt: 4 min-w: 0',
      vertical: 'pt: 0 ps: 4 min-w: 0',
    }
  }
</scope>
<div sx="pt: 4 min-w: 0"><slot></slot></div>
