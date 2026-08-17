<!--
  <label> — size match; weight + transform tokens; .req gate via group-required when.
  Host + .req via style chains / match / when.
  Tokens: foreground, type-label-weight, type-label-case. Default type is sm, not base.
-->

<scope>
  props: {
    size: 'md' | 'sm' | 'lg' = 'md'
    required = false
  }

  style base = {fg(foreground).cursor(pointer).group().type({ family: body, weight: bind(labelWeight), tracking: normal, size: sm, case: type-label-case })}
  style sizeMd = {type({ size: sm })}
  style sizeSm = {type({ size: xs })}
  style sizeLg = {type({ size: md })}
  style req = {display(none).margin({ s: 1 }).fg(danger).type({ weight: bold })}
</scope>

<label style(base, { match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg } } })>
  <slot></slot>
  <span hidden="true" style(req, {when: {group-required: display(inline)}})>*</span>
</label>
