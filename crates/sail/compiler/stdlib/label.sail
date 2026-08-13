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

  style base = {paint: ink({ color: foreground }).cursor(pointer).group(), text: type({ family: body, weight: bind(labelWeight), tracking: normal, size: base })}
  style sizeMd = {text: type({ size: base })}
  style sizeSm = {text: type({ size: sm })}
  style sizeLg = {text: type({ size: lg })}
  style req = {box: flow({ display: none }).space({ ms: 1 }), paint: ink({ color: destructive }), text: type({ weight: bold })}
</scope>

<label style(base, { match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg } } })>
  <slot></slot>
  <span hidden="true" style(req, {when: {group-required: {box: flow({ display: inline })}}})>*</span>
</label>
