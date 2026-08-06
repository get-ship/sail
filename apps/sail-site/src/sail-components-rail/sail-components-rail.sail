<!--
  <sail-components-rail> — /components/* layout (no sidebar).

  Parent is site-header <main> with overflow:hidden + a definite flex height.
  This shell MUST fill that budget (h(full)) and own the only scrollport.
  A bare <main grow overflowY auto> under a non-flex parent expands to content
  height → parent clips → nothing scrolls. Same geometry as sail-docs-rail.
-->

<!-- Fill site-header appMain; independent catalog scrollport. -->
<row
  nowrap
  align="stretch"
  w="full"
  grow
  min-w="0"
  gap="0"
  components-shell

>
  style.minH(0).h(full).minW(0).overflow(hidden)

  <main
    grow
    min-w="0"
    w="full"
    components-scroll

  >
    style.basis(0).minH(0).minW(0).h(full).overflowY(auto).overflowX(hidden)

    <slot name="route"></slot>
  </main>
</row>
