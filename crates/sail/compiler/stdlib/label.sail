<!--
  <label> — size match; weight token; .req gate via group-required when.
  Paint (Style product only): host + .req via style chains / match / when.
  Tokens: component.label (cssNames label-foreground, label-required) + label-weight.
-->

<scope>
  props: {
    size: 'md' | 'sm' | 'lg' = 'md'
    required = false
  }

  const base = style.fontFamily(body).fontWeight(bind(labelWeight)).tracking(normal).fg(label-foreground).cursor(pointer).fontSize(base).group()
  const sizeMd = style.fontSize(base)
  const sizeSm = style.fontSize(sm)
  const sizeLg = style.fontSize(sm)
  const req = style.display(none).fg(label-required).fontWeight(bold).ms(1)
</scope>

<label sx={[
  base,
  match(size, {
    md: sizeMd,
    sm: sizeSm,
    lg: sizeLg,
  }),
]}>
  <slot></slot>
  <span aria-hidden="true" sx={[
    req,
    when({
      group-required: style.display(inline),
    }),
  ]}>*</span>
</label>
