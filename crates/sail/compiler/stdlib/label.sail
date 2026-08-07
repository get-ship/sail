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

  const base = style({ paint: fg(label-foreground).cursor(pointer).group(), text: fontFamily(body).fontWeight(bind(labelWeight)).tracking(normal).fontSize(base) })
  const sizeMd = style({ text: fontSize(base) })
  const sizeSm = style({ text: fontSize(sm) })
  const sizeLg = style({ text: fontSize(sm) })
  const req = style({ box: display(none).ms(1), paint: fg(label-required), text: fontWeight(bold) })
</scope>

<label style([
  base,
  match(size, {
    md: sizeMd,
    sm: sizeSm,
    lg: sizeLg,
  }),
])>
  <slot></slot>
  <span hidden="true" style([
    req,
    when({
      group-required: style({ box: display(inline) }),
    }),
  ])>*</span>
</label>
