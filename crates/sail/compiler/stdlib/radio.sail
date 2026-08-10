<!--
  <radio> — label + native radio. Root stays <label> for native semantics.
  Host label ink from component.radio.fg (--radio-foreground). Control face uses
  free foreground/background (checked → primary); inset disc on checked is
  RADIO_INPUT_MAP (box-shadow, not Style vocab). value + disabled are host props;
  radio-group composition wires shared name, checked, and input.disabled (out of
  this PE's closed action surface).
-->

<scope>
  props: {
    value = ''
    disabled = false
  }

  // Host when:disabled does not paint — :disabled/:aria-disabled only match form
  // controls; label hosts carry [disabled] for radio-group to read. Input when:
  // disabled paints after group (or native) sets input.disabled.
  style host = {
    paint: fg(radio.foreground).cursor(pointer).select(none),
    text: fontFamily(body).fontSize(sm),
  }
  // Control face: free foreground/background ring + primary when checked
  // (not field.* — D1c; pack owns radio-foreground label ink only).
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
