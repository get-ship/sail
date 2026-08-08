<!--
  <radio-group> — L1 language element (element mode). A vertical group of
  <radio> options. Bare tag lowers to a layout column (`.sail-radio-group`).
  Selection, arrow-key roving, focus, and form participation are the BROWSER's:
  use:radioGroup gives every child radio's <input> the shared name (so native
  single-select + arrow navigation kick in), reflects the group's `value` to the
  checked radio, writes a hidden <input> for the form, and invokes onChange.
  disabled / aria-label / required propagate from the group.

  Paint (Style product): layout owns stack+gap (`<column gap="2">`); typeface is Style.
  tone/size remain ordinary props (stamped / behavior); no CVA paint maps.
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

<column gap="2" use:radioGroup style(host)><slot></slot></column>
