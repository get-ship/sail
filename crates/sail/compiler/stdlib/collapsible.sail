<!--
  <collapsible> — L1 language element (element mode). A native disclosure: a
  <details>/<summary> pair. Open/close toggle, Enter/Space activation, and the
  ARIA expanded state are the BROWSER's — none of it is hand-rolled. Named slots
  (trigger inside <summary> + the default content slot) project at compile time.
  Bare tag lowers to <details class="sail-collapsible">.

  Behavior rides stacked use: behaviors, each a reusable mix-in the normal
  use: pipeline lowers to runAction at the use site:
    use:detailsToggle   — re-emit native `toggle` as composed `sail-toggle` {open}
    use:detailsDisabled — suppress click + Enter/Space while [disabled]
                          (native <details> has no disabled concept)
  Open state is the native `open` attribute; the animated open/close uses
  ::details-content + interpolate-size (degrades to instant where unsupported).
  Faithful to the sail-collapsible twin's native rebuild.
-->

<scope>
  props: {
    open = false
    disabled = false
  }
</scope>

<details use:detailsToggle use:detailsDisabled sx="block font-body">
  <summary class="trigger-wrap"><slot name="trigger"></slot></summary>
  <div class="content-wrap"><div class="content-inner" sx="px-4 py-3"><slot></slot></div></div>
</details>
