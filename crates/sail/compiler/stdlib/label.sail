<!--
  <label> — size CVA; weight token; .req gate in element-skins anatomy.
  Variance = CVA-in-props axes + optional compound only. size is an ordinary
  prop/axis, not a language channel.
-->
<scope>
  props: {
    size: {
      md: 'font-size: base',
      sm: 'font-size: sm',
      lg: 'font-size: sm',
    }
    required = false
  }
</scope>
<label sx="font-family: body font: (--label-weight) tracking: normal color: foreground cursor: pointer font-size: base group">
  <slot></slot>
  <span class="req" aria-hidden="true" sx="hidden group-required:inline color: destructive font-weight: bold ms: 1">*</span>
</label>
