<!--
  <toggle-group> — segmented Style match; structural orientation stamp on cells.
  Size/variant paint is authored on each toggle.
-->

<scope>
  props: {
    type: 'single' | 'multiple' = 'single'
    value = ''
    variant: 'outline' | 'ghost' | 'segmented' = 'outline'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    onToggleChange: (detail: { value: string }) => void
  }

  style segmented = {pad(1).bg(muted).border({ color: border, width: 1 }).radius(md)}
</scope>

<row inline gap="0" use:toggleGroup style({match: {variant: {outline: bg(transparent), ghost: bg(transparent), segmented: segmented}}})>
  <slot></slot>
</row>
