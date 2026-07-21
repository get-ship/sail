<!--
  <radio-group> — L1 language element (element mode). A vertical group of
  <radio> options. Bare tag lowers to a layout column (`.sail-radio-group`).
  Selection, arrow-key roving, focus, and form participation are the BROWSER's:
  use:radioGroup gives every child radio's <input> the shared name (so native
  single-select + arrow navigation kick in), reflects the group's `value` to the
  checked radio, writes a hidden <input> for the form, and dispatches sail-change.
  disabled / aria-label / required propagate from the group. Faithful to the
  sail-radio-group twin, rebuilt on native radios.

  Host paint: layout owns stack+gap (`<column gap="2">`); typeface is sx
  (`font-body`). No author `<style>` — first stdlib element at zero raw skin.
-->

<scope>
  props: {
    value = ''
    disabled = false
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'sm' | 'md' | 'lg' = 'md'
    name = ''
    label = ''
    required = false
  }
</scope>

<column gap="2" sx="font-body" use:radioGroup role="radiogroup"><slot></slot></column>
