<!--
  <tab-list> — strip chrome. Sticky + variant CVA. Orientation structural stamp from use:tabs.
  Paint: strip/rail from component.tab-list tokens
         (cssNames tab-list, tab-list-border, tab-list-rail).
-->
<scope>
  props: {
    sticky = false
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    variant: {
      line: 'border-b border: tab-list-border',
      pill: 'p: 1 border: 0 rounded: md bg: tab-list-rail border-b: 0',
      enclosed: 'border-b border: tab-list-border',
    }
  }
  compound: [
    { when: { sticky: true }, sx: 'sticky top: 0 z: (--z-sticky)' },
    { when: { orientation: 'vertical' }, sx: 'border-b: 0 border-e border: tab-list-border' },
    { when: { orientation: 'vertical', variant: 'pill' }, sx: 'border-e: 0' },
  ]
</scope>
<row align="stretch" gap="1" role="tablist" sx="border-b border: tab-list-border bg: tab-list"><slot></slot></row>
