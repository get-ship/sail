<!--
  <color> — system color swatch product. Impl: native input type=color.
  One field border, edge-to-edge fill (UA rims cleared in COLOR_MAP).
  Not sail-color-picker (full channel UI package). value · name · disabled · invalid.
-->

<scope>
  props: {
    value = '#000000'
    disabled = false
    invalid = false
    name = ''
  }

  // One field border only; swatch is edge-to-edge inside it.
  style color = {
    box: w(9).h(control-h-md).pad(0).overflow(hidden).minH(0),
    paint: appearance(none).borderWidth(1).border(solid).border(field.border).radius(md).bg(field).cursor(pointer),
    when: {
      hover: { paint: border(border-strong) },
      disabled: { paint: cursor(not-allowed).opacity(50) },
    },
  }
</scope>

<input
  type="color"
  style(color, {
    compound: [{ invalid: true, paint: border(destructive) }],
  })
>
