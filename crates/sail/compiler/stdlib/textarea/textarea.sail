<!--
  <textarea> — native multi-line field. Style product only.
  Shares the field surface with <input>: bg(input) / fg(foreground) / border.
  `rows` is the native HTML rows attr (default 3). size is pad/type scale.
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    surface: 'default' | 'filled' | 'flushed' = 'default'
    invalid = false
    disabled = false
    placeholder = ''
    rows = 3
  }

  // Same input / foreground / border spend as <input> — no special well.
  style base = {display(block).size({ min-h: control-h-md }).pad({ y: 1_5, x: 3 }).fg(foreground).bg(input).border({ color: border, width: 1 }).radius(md).elevation(xs).type({ size: base }).boxSizing(border), when: {hover: border({ color: border-strong }), disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  style sizeXs = {size({ min-h: control-h-2xs }).pad({ x: 2 }).type({ size: xs })}
  style sizeSm = {size({ min-h: control-h-sm }).pad({ x: 2_5 }).type({ size: sm })}
  style sizeMd = {size({ min-h: control-h-md }).pad({ x: 3 }).type({ size: base })}
  style sizeLg = {size({ min-h: control-h-lg }).pad({ x: 4 }).type({ size: md })}
  // default = current field (bg input, border). filled = muted well, no outer border.
  // flushed = no fill, underline only (border edge b).
  style surfaceDefault = {bg(input).border({ color: border, width: 1 })}
  style surfaceFilled = {bg(muted).border({ color: transparent }).elevation(none), when: {hover: border({ color: transparent })}}
  style surfaceFlushed = {bg(transparent).elevation(none).radius(none).border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 1, edge: b, color: border }), when: {hover: border({ color: border-strong, edge: b })}}
</scope>

<textarea
  style(base, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg }, surface: { default: surfaceDefault, filled: surfaceFilled, flushed: surfaceFlushed } }, compound: [{invalid: true, border({ color: destructive })}]})
>
