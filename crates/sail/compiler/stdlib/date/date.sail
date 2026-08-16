<!--
  <date> — temporal picker product. Impl: native input; type prop selects
  precision (date | time | datetime-local | month | week). Shared field surface.
  Platform calendar/time UI. min · max · step · name · required · readonly · value.
-->

<scope>
  props: {
    type: 'date' | 'time' | 'datetime-local' | 'month' | 'week' = 'date'
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    invalid = false
    disabled = false
    placeholder = ''
    name = ''
    required = false
    readonly = false
    value = ''
    min = ''
    max = ''
    step = ''
  }

  style base = {display(block).size({ min-h: control-h-md }).pad({ y: 1_5, x: 3 }).fg(foreground).bg(input).border({ color: border, width: 1 }).radius(md).elevation(xs).type({ size: base }), when: {hover: border({ color: border-strong }), disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  style sizeXs = {size({ min-h: control-h-2xs }).pad({ x: 2 }).type({ size: xs })}
  style sizeSm = {size({ min-h: control-h-sm }).pad({ x: 2_5 }).type({ size: sm })}
  style sizeMd = {size({ min-h: control-h-md }).pad({ x: 3 }).type({ size: base })}
  style sizeLg = {size({ min-h: control-h-lg }).pad({ x: 4 }).type({ size: md })}
</scope>

<input
  type="date"
  style(base, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [{invalid: true, border({ color: destructive })}]})
>
