<!--
  <radio> — label + native radio. Root stays <label> for native semantics.
  Nested input paint matches freeform <input type=radio> (input.sail); inset
  disc on checked is RADIO_INPUT_MAP (box-shadow, not Style vocab).
-->

<scope>
  props: {
    value = ''
    disabled = false
  }

  const host = style.fontFamily(body).fontSize(sm).fg(radio-foreground).cursor(pointer).select(none)
  const input = style.appearance(none).display(inline-block).align(middle).m(0).pad(0).minH(0).w(icon-size-sm).h(icon-size-sm).fontSize(base).borderWidth(2).border(foreground).bg(background).shadow(xs).radius(full).cursor(pointer).when({
    checked: style.bg(primary).border(primary),
    disabled: style.cursor(not-allowed).opacity(50),
  })
</scope>

<label>
  style(host)

  <input type="radio" shrink="0">
    style(input)
  </input><span><slot></slot></span>
</label>
