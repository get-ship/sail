<!--
  <radio-group> — vertical group of <radio> options. use:radioGroup wires shared
  name, value reflection, form hidden input, and onChange. Host is layout + typeface.
-->

<scope>
  props: {
    value = ''
    disabled = false
    name = ''
    label = ''
    required = false

    onChange: (detail: { value: string }) => void
  }

  style host = {text: type({ family: body })}
</scope>

<column gap="2" use:radioGroup style(host)>
  <slot></slot>
</column>
