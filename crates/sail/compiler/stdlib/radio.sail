<!--
  <radio> — label + native radio. Root stays <label> for native semantics.
  Layout flex/gap is contracts (cannot put <row> on label). Disabled dim is
  contracts too ([disabled] attr; disabled: prefix only matches :disabled/aria-disabled).
-->

<scope>
  props: {
    value = ''
    disabled = false
  }
</scope>

<label sx="font-family: body font-size: sm color: foreground cursor: pointer select: none">
  <input type="radio" sx="shrink: 0 m: 0" /><span class="rlabel"><slot></slot></span>
</label>
