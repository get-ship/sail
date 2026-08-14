<!--
  <label> — size match; weight token; .req gate via group-required when.
  Host + .req via style chains / match / when.
  Tokens: foreground + label-weight.
-->

<scope>
  props: {
    size: 'md' | 'sm' | 'lg' = 'md'
    required = false
  }

  style base = {fg(foreground).cursor(pointer).group().type({ family: body, weight: bind(labelWeight), tracking: normal, size: base })}
  style sizeMd = {type({ size: base })}
  style sizeSm = {type({ size: sm })}
  style sizeLg = {type({ size: lg })}
  style req = {display(none).margin({ s: 1 }).fg(destructive).type({ weight: bold })}
</scope>

<label style(base, { match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg } } })>
  <slot></slot>
  <span hidden="true" style(req, {when: {group-required: display(inline)}})>*</span>
</label>
