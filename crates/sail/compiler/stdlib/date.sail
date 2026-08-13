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

  style base = {box: flow({ display: block }).size({ min-h: control-h-md, w: full }).space({ pad-y: 1_5, pad-x: 3 }), paint: ink({ color: foreground }).fill({ color: input }).border({ color: border, width: 1 }).radius(md).elevation(xs), text: type({ size: base }), when: {hover: {paint: border({ color: border-strong })}, disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
  style sizeXs = {box: size({ min-h: control-h-2xs }).space({ pad-x: 2 }), text: type({ size: xs })}
  style sizeSm = {box: size({ min-h: control-h-sm }).space({ pad-x: 2_5 }), text: type({ size: sm })}
  style sizeMd = {box: size({ min-h: control-h-md }).space({ pad-x: 3 }), text: type({ size: base })}
  style sizeLg = {box: size({ min-h: control-h-lg }).space({ pad-x: 4 }), text: type({ size: md })}
</scope>

<input
  type="date"
  style(base, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [{invalid: true, paint: border({ color: destructive })}]})
>
