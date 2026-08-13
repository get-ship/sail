<!--
  <textarea> — native multi-line field. Style product only.
  Shares the field surface with <input>; residual no longer carries field paint.
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    invalid = false
    disabled = false
    placeholder = ''
    rows = 3
  }

  // Shares component.field surface with input/select (global fallbacks in the pack).
  style base = {box: flow({ display: block }).size({ min-h: control-h-md, w: full }).space({ pad-y: 1_5, pad-x: 3 }), paint: ink({ color: field.foreground }).fill({ color: field }).border({ color: field.border, width: 1 }).radius(md).elevation(xs), text: type({ size: base }), when: {hover: {paint: border({ color: border-strong })}, disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
  style sizeXs = {box: size({ min-h: control-h-2xs }).space({ pad-x: 2 }), text: type({ size: xs })}
  style sizeSm = {box: size({ min-h: control-h-sm }).space({ pad-x: 2_5 }), text: type({ size: sm })}
  style sizeMd = {box: size({ min-h: control-h-md }).space({ pad-x: 3 }), text: type({ size: base })}
  style sizeLg = {box: size({ min-h: control-h-lg }).space({ pad-x: 4 }), text: type({ size: md })}
</scope>

<textarea
  style(base, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [{invalid: true, paint: border({ color: destructive })}]})
>
