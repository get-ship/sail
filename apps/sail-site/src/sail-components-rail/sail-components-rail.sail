<!--
  <sail-components-rail> — mounted by routes/components/_layout.sail for
  every /components/* route. The catalog renders full-screen (no sidebar) —
  the operator's "no nested nav bars" rule: one product switcher + one
  sidebar, never stacked, and Components has none.
-->

<!-- Catalog fills site-header <main>; own scrollport (no nested rail). -->
<main grow min-w="0" w="full" sx={style.basis(0).minH(0).minW(0).overflowY(auto).overflowX(hidden).padX(0).padY(0)}>
  <slot name="route"></slot>
</main>
