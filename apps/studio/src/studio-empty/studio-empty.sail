<!--
  <studio-empty> — bordered empty-state shell for workbench lists.
  Props: title, variant, size. Slots: icon, action, default (body).
-->
<scope>
  props: {
    title = ''
    variant: 'default' | 'inline' | 'placeholder' | 'connecting' = 'placeholder'
    size: 'md' | 'sm' | 'lg' = 'sm'
  }
  style emptyRoot = { size(max).pad(3).border(subtle).radius(md) }
</scope>

<empty-state studio-empty variant={variant()} size={size()} style(emptyRoot)>
  <slot name="icon"></slot>
  <strong slot="title">{title()}</strong>
  <slot></slot>
  <slot name="action"></slot>
</empty-state>
