<!--
  <tab-list> — strip chrome. Sticky + variant CVA. Orientation stamped by use:tabs.
-->
<scope>
  props: {
    sticky = false
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    variant: {
      line: 'border-b border-border',
      pill: 'p-1 border-0 rounded-md bg-muted border-b-0',
      enclosed: 'border-b border-border',
    }
  }
  compound: [
    { when: { sticky: true }, sx: 'sticky top-0 z-(--z-sticky)' },
    { when: { orientation: 'vertical' }, sx: 'border-b-0 border-e border-border' },
    { when: { orientation: 'vertical', variant: 'pill' }, sx: 'border-e-0' },
  ]
</scope>
<row align="stretch" gap="1" role="tablist" sx="border-b border-border bg-background"><slot></slot></row>
