<!--
  <label> — L1 language element (element mode). Bare tag: it lowers away at
  compile time to a native `<label>`; the skin routes to the framework layer
  keyed on label; the contract feeds the element manifest.

  Faithful to the feat/next sail-label baseline for `size` and `required`.
  `for` is deliberately NOT a declared prop — the baseline forwards it to an
  inner native `<label>` through controller JS (`labelEl.htmlFor = ...`) —
  that indirection is a LANG-GAP that dissolves at L1. Here the root IS a
  real `<label>`, so a use-site `for="…"` attribute merges straight onto it
  through the ordinary use-site attr-merge; no controller, no prop needed.

  Host paint: sx owns typeface/color/cursor/default size; framework host-display
  supplies inline-block for the native label root. Size sm/lg share text-sm
  (baseline). Anatomy `.req` stays author CSS (child gate).
-->

<scope>
  props: {
    size: 'sm' | 'md' | 'lg' = 'md'
    required = false
  }
</scope>

<label sx="font-body tracking-normal text-foreground cursor-pointer text-base">
  <slot></slot>
  <span class="req" aria-hidden="true">*</span>
</label>
