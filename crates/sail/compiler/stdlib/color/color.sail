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
  style color = {size({ w: 9, h: control-h-md, min-h: 0 }).pad(0).overflow({ all: hidden }).appearance(none).border({ color: border, width: 1, style: solid }).radius(md).bg(control).cursor(pointer), when: {hover: border({ color: border-strong }), disabled: cursor(not-allowed).filter({ opacity: 50 })}}
</scope>

<input
  type="color"
  style(color, {compound: [{invalid: true, border({ color: destructive })}]})
>
