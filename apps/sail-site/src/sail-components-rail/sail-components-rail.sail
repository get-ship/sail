<!--
  <sail-components-rail> — /components/* layout (no sidebar).

  Parent is site-header <main> with overflow:hidden + a definite flex height.
  This shell MUST fill that budget (h(full)) and own the only scrollport.
  A bare <main grow overflowY auto> under a non-flex parent expands to content
  height → parent clips → nothing scrolls. Same geometry as sail-docs-rail.
-->

<!-- Fill site-header appMain; independent catalog scrollport. -->
<row nowrap align="stretch" gap="0" components-shell>
  style({  box: w(full).grow(1).minW(0).minH(0).h(full).overflow(hidden)  })

  <main components-scroll>
    style({  box: grow(1).minW(0).w(full).basis(0).minH(0).h(full).overflowY(auto).overflowX(hidden)  })

    <slot name="route"></slot>
  </main>
</row>
