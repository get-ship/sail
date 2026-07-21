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

  The baseline's host/inner-label split (host `:host{display:inline-block}`
  wrapping an inner `label{display:inline; font:…}`) also dissolves: L1 has
  one node, so this file keeps the host's outward `display: inline-block`
  (what a page author placing <sail-label> actually saw) and folds the
  inner label's font/color rules onto the same selector.

  The `required` marker is static structure (a span, always present) with
  its visibility toggled by CSS on the `required` attribute — the baseline's
  `{#if required()}…{/if}` is template control flow, which element mode
  rejects (components have control flow; elements are static structure).
-->

<scope>
  props: {
    size: 'sm' | 'md' | 'lg' = 'md'
    required = false
  }
  variants: {
    size: {
      sm: 'text-sm',
      lg: 'text-sm',
    }
  }
</scope>

<style>
  label { font-weight: var(--label-weight); }
  label[hidden] { display: none; }

  .req {
    display: none;
    color: var(--destructive);
    margin-left: var(--space-1);
    font-weight: var(--font-weight-bold);
  }
  label[required] .req { display: inline; }
</style>

<label sx="inline-block font-body tracking-normal text-foreground cursor-pointer text-base">
  <slot></slot>
  <span class="req" aria-hidden="true">*</span>
</label>
