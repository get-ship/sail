<!--
  <radio-group> — vertical group of <radio> options. use:radioGroup wires shared
  name, value reflection, form hidden input, and onChange. Host is layout + typeface.
-->

<scope>
  props: {
    value = ''
    disabled = false
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    size: 'sm' | 'md' | 'lg' = 'md'
    name = ''
    label = ''
    required = false

    onChange: (detail: { value: string }) => void
  }

  style host = { text: fontFamily(body) }
</scope>

<column gap="2" use:radioGroup style(host)>
  <slot></slot>
</column>
