<!--
  <sail-components-rail> — mounted by routes/components/_layout.sail for
  every /components/* route. The catalog renders full-screen (no sidebar) —
  the operator's "no nested nav bars" rule: one product switcher + one
  sidebar, never stacked, and Components has none.
-->

<main sx="grow-1 min-w-0">
  <slot name="route"></slot>
</main>
