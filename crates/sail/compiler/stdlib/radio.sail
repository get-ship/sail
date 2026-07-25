<!--
  <radio> — label + native radio. Root stays <label> for native semantics.
  Layout flex/gap is contracts (cannot put <row> on label). Disabled dim is
  contracts too ([disabled] attr; disabled: prefix only matches :disabled/aria-disabled).
  Paint (Style product): host type from component.radio tokens (cssName radio-foreground).
-->

<scope>
  props: {
    value = ''
    disabled = false
  }

  const host = style.fontFamily(body).fontSize(sm).fg(radio-foreground).cursor(pointer).select(none)
  const input = style.shrink(0).m(0)
</scope>

<label sx={host}>
  <input type="radio" sx={input} /><span class="rlabel"><slot></slot></span>
</label>
