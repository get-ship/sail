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
  style color = {box: size({ w: 9, h: control-h-md, min-h: 0 }).space({ pad: 0 }).overflow({ all: hidden }), paint: appearance(none).border({ color: field.border, width: 1, style: solid }).radius(md).fill({ color: field }).cursor(pointer), when: {hover: {paint: border({ color: border-strong })}, disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
</scope>

<input
  type="color"
  style(color, {compound: [{invalid: true, paint: border({ color: destructive })}]})
>
