<!--
  <collapsible> — native disclosure. Behavior: use:detailsToggle + use:detailsDisabled.
  Style product: host type + content pad; disabled dims via compound.
-->

<scope>
  props: {
    open = false
    disabled = false
  }

  const host = style.display(block).fontFamily(body)
  const content = style.padX(4).padY(3)
</scope>

<details use:detailsToggle use:detailsDisabled sx={[
  host,
  compound({ disabled: true }, style.opacity(50).cursor(not-allowed)),
]}>
  <summary class="trigger-wrap"><slot name="trigger"></slot></summary>
  <div class="content-wrap"><div class="content-inner" sx={content}><slot></slot></div></div>
</details>
