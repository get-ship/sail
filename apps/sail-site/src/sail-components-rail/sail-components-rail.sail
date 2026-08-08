<!--
  /components/* layout shell. Fills site-header main and owns the catalog scrollport
  (parent is overflow-hidden with a definite flex height — shell must h(full)).
-->
<scope>
  style shell = { box: w(full).grow(1).minW(0).minH(0).h(full).overflow(hidden) }
  style scrollMain = { box: grow(1).minW(0).w(full).basis(0).minH(0).h(full).overflowY(auto).overflowX(hidden) }
</scope>

<row nowrap align="stretch" gap="0" style(shell)>
  <main style(scrollMain)>
    <slot name="route"></slot>
  </main>
</row>
