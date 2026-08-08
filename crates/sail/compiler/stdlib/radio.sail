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

  style host = {
    paint: fg(radio.foreground).cursor(pointer).select(none),
    text: fontFamily(body).fontSize(sm),
  }
  // Control face shares field surface with freeform <input type=radio>
  // (field.foreground border so the ring stays visible on muted hosts).
  style input = {
    box: display(inline-block).m(0).pad(0).minH(0).w(icon-size-sm).h(icon-size-sm),
    paint: appearance(none).align(middle).borderWidth(2).border(solid).border(foreground).bg(background).shadow(xs).radius(full).cursor(pointer),
    text: fontSize(base),
    when: {
      checked: { paint: bg(primary).border(primary) },
      disabled: { paint: cursor(not-allowed).opacity(50) },
    },
  }
</scope>

<label style(host)>
  <input type="radio" style(input, { box: shrink(0) })>
  <span><slot></slot></span>
</label>
