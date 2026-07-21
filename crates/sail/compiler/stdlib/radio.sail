<!--
  <radio> — L1 language element (element mode). A single radio option: a native
  <input type="radio"> (skinned by the framework's control contracts) + a label.
  Bare tag lowers to <label class="sail-radio">. Used inside <radio-group>, which
  wires the shared name, checked state, and value via use:radioGroup.

  Host paint: typeface/color/cursor/select on sx. Layout (inline-flex + gap +
  align) stays author CSS — root must remain <label> (not <row>) for native
  label semantics. Disabled opacity residue.
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<label sx="font-body text-sm text-foreground cursor-pointer select-none">
  <input type="radio" /><span class="rlabel"><slot></slot></span>
</label>
