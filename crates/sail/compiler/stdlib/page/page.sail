<!--
  <page> — size match owns cell geometry (author size on each page).
  Paint from foreground / hover-bg / muted.
-->

<scope>
  props: {
    value = ''
    disabled = false
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  style base = {pad({ x: 2 }).fg(foreground).bg(transparent).radius(md).border({ color: transparent, width: 1 }).cursor(pointer).select(none).type({ family: body, weight: medium }), animate: { transition: play(colors, fast) }}
  style sizeMd = {size({ min-w: control-h-md, h: control-h-md }).type({ size: sm })}
  style sizeSm = {size({ min-w: control-h-sm, h: control-h-sm }).type({ size: xs })}
  style sizeLg = {size({ min-w: control-h-lg, h: control-h-lg }).type({ size: base })}
</scope>

<button
  type="button"
  style(base, {when: {hover: bg(hover-bg), current: bg(muted).fg(foreground).border({ color: muted }), disabled: filter({ opacity: 50 }).cursor(not-allowed), focus: border({ style: solid, kind: outline, width: 2, color: ring })}, match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg } }})
>
  <slot></slot>
</button>
