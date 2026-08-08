<!--
  <toggle-group> — segmented Style match; structural orientation stamp on cells only.
  Size/variant paint is authored on each toggle.
  Paint: segmented rail from component.toggle-group tokens
         (cssNames toggle-group, toggle-group.border).
-->

<scope>
  props: {
    type: 'single' | 'multiple' = 'single'
    value = ''
    variant: 'outline' | 'ghost' | 'segmented' = 'outline'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  
    onToggleChange: (detail: { value: string }) => void
}

  style segmented = { box: pad(1), paint: bg(toggle-group).borderWidth(1).border(toggle-group.border).radius(md) }
</scope>

<row inline gap="0" use:toggleGroup style({ match: { variant: { outline: { paint: bg(transparent) }, ghost: { paint: bg(transparent) }, segmented: segmented } } })>
  <slot></slot>
</row>
