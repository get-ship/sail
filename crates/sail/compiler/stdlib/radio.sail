<!--
  <radio> — label + native radio. Root stays <label> for native semantics.
  Host label ink from foreground. Control face
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
  style host = {paint: ink({ color: foreground }).cursor(pointer).select(none), text: type({ family: body, size: sm, weight: normal, case: normal, tracking: normal })}
  // Checked disc via RADIO_INPUT_MAP (empty face + primary dot — native, no SVG).
  style input = {box: flow({ display: inline-block, shrink: 0 }).space({ m: 0, pad: 0 }).size({ min-h: 0, w: icon-size-md, h: icon-size-md }), paint: appearance(none).align(middle).border({ color: border, width: 1, style: solid }).radius(full).cursor(pointer).boxSizing(border), when: {disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
</scope>

<label style(host)>
  <input type="radio" style(input, {box: flow({ shrink: 0 })})>
  <span><slot></slot></span>
</label>
