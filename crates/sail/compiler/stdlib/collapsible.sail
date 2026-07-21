<!--
  <collapsible> — native disclosure. Behavior: use:detailsToggle + use:detailsDisabled.
  Paint: host sx; disabled host dims via compound ([disabled] attr — not :disabled).
-->

<scope>
  props: {
    open = false
    disabled = false
  }
  compound: [
    { when: { disabled: true }, sx: 'opacity-50 cursor-not-allowed' },
  ]
</scope>

<details use:detailsToggle use:detailsDisabled sx="block font-body">
  <summary class="trigger-wrap"><slot name="trigger"></slot></summary>
  <div class="content-wrap"><div class="content-inner" sx="px-4 py-3"><slot></slot></div></div>
</details>
