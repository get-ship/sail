<!--
  <studio-empty> — empty placeholder composition over empty-state.
  Props: title, variant, size. Slots: icon, action, default (body).
  Ban-clean: L1 empty-state only.
-->
<scope>
  props: {
    title = ''
    variant: 'default' | 'inline' | 'placeholder' | 'connecting' = 'placeholder'
    size: 'md' | 'sm' | 'lg' = 'sm'
  }
</scope>

<empty-state
  data-studio-empty
  variant={variant()}
  size={size()}
  w="full"
  sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}
>
  <slot name="icon"></slot>
  <strong slot="title">{title()}</strong>
  <slot></slot>
  <slot name="action"></slot>
</empty-state>
