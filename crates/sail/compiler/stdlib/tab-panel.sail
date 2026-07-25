<!--
  <tab-panel> — orientation Style match (stamped by use:tabs).
-->
<scope>
  props: {
    name = ''
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }

  const base = style.minW(0)
  const horizontal = style.padT(4).minW(0)
  const vertical = style.padT(0).padS(4).minW(0)
</scope>
<div sx={[
  base,
  match(orientation, {
    horizontal: horizontal,
    vertical: vertical,
  }),
]}><slot></slot></div>
