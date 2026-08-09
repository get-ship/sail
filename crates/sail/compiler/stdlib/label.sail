<!--
  <label> — size match; weight token; .req gate via group-required when.
  Host + .req via style chains / match / when.
  Tokens: component.label (label.foreground, label.required) + label-weight.
-->

<scope>
  props: {
    size: 'md' | 'sm' | 'lg' = 'md'
    required = false
  }

  style base = { paint: fg(label.foreground).cursor(pointer).group(), text: fontFamily(body).fontWeight(bind(labelWeight)).tracking(normal).fontSize(base) }
  style sizeMd = { text: fontSize(base) }
  style sizeSm = { text: fontSize(sm) }
  style sizeLg = { text: fontSize(lg) }
  style req = { box: display(none).ms(1), paint: fg(label.required), text: fontWeight(bold) }
</scope>

<label style(base, { match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg } } })>
  <slot></slot>
  <span hidden="true" style(req, { when: { group-required: { box: display(inline) } } })>*</span>
</label>
