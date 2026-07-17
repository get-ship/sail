<!--
  <collapsible> — L1 language element (element mode). A native disclosure: a
  <details>/<summary> pair. Open/close toggle, Enter/Space activation, and the
  ARIA expanded state are the BROWSER's — none of it is hand-rolled. Named slots
  (trigger inside <summary> + the default content slot) project at compile time.
  Bare tag lowers to <details class="sail-collapsible">.

  Behavior rides use:collapsible: re-emit the native `toggle` as `sail-toggle`
  {open}, and suppress activation while [disabled] (native <details> has no
  disabled concept — the one bit native HTML can't express). Open state is the
  native `open` attribute; the animated open/close uses ::details-content +
  interpolate-size (degrades to instant where unsupported). Faithful to the
  sail-collapsible twin's native rebuild.
-->

<scope>
  props: {
    open = false
    disabled = false
  }
</scope>

<style>
  /* The framework's global :where(summary) / :where(details) treatment in
     contracts.css already supplies the flex summary, the rotating chevron
     (::after), the open-state header, and the ::details-content open/close
     animation — collapsible inherits all of it. This skin adds only what is
     element-specific: identity + the disabled state (native <details> has no
     disabled concept) + the content padding hook. */
  .sail-collapsible {
    display: block;
    font-family: var(--font-body);
  }
  .sail-collapsible[disabled] summary { cursor: not-allowed; opacity: 0.5; }
  .sail-collapsible .content-inner { padding: var(--space-3) var(--space-4); }
</style>

<details use:collapsible>
  <summary class="trigger-wrap"><slot name="trigger"></slot></summary>
  <div class="content-wrap"><div class="content-inner"><slot></slot></div></div>
</details>
