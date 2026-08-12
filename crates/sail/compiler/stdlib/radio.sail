<!--
  <radio> — label + native radio. Root stays <label> for native semantics.
  Host label ink from component.radio.fg (--radio-foreground). Control face
  aligns with <checkbox> (foreground/background ring, primary when checked).
  JS-free for toggle; radio-group supplies shared name/value/onChange.
  value + disabled are host props.
-->

<scope>
  props: {
    value = ''
    disabled = false
  }

  // Host when:disabled does not paint — :disabled/:aria-disabled only match form
  // controls; label hosts carry [disabled] for radio-group to read. Input when:
  // disabled paints after group (or native) sets input.disabled.
  // textCase(normal) → CSS none: host is a <label>; label-transform must not scream.
  style host = {
    paint: fg(radio.foreground).cursor(pointer).select(none),
    text: fontFamily(body).fontSize(sm).fontWeight(normal).textCase(normal).tracking(normal),
  }
  // Fill + centered inset via RADIO_INPUT_MAP (host-specific).
  // Match checkbox face: soft 1px border, no shadow, border-box outer size.
  style input = {
    box: display(inline-block).m(0).pad(0).minH(0).w(icon-size-md).h(icon-size-md).shrink(0),
    paint: appearance(none).align(middle).borderWidth(1).border(solid).border(border).radius(full).cursor(pointer).boxSizing(border),
    when: {
      disabled: { paint: cursor(not-allowed).opacity(50) },
    },
  }
</scope>

<label style(host)>
  <input type="radio" style(input, { box: shrink(0) })>
  <span><slot></slot></span>
</label>
