<!--
  <sail-components-rail> — mounted by routes/components/_layout.sail for
  every /components/* route. The catalog renders full-screen (no sidebar) —
  the operator's "no nested nav bars" rule: one product switcher + one
  sidebar, never stacked, and Components has none.
-->

<main sx={style.grow(1).basis(0).minW(0).minH(0).w(full).overflowX(hidden)}>
  <slot name="route"></slot>
</main>
