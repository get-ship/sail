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

  style segmented = {box: space({ pad: 1 }), paint: fill({ color: toggle-group }).border({ color: toggle-group.border, width: 1 }).radius(md)}
</scope>

<row inline gap="0" use:toggleGroup style({match: {variant: {outline: {paint: fill({ color: transparent })}, ghost: {paint: fill({ color: transparent })}, segmented: segmented}}})>
  <slot></slot>
</row>
