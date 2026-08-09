<!--
  <sail-fill-shell> — full-bleed fill shell for workbench / residual routes.
  Used by /playground, parked /sail/tutorial*, and legacy redirect prefixes
  that must not carry a docs rail.
-->
<scope>
  style shell = { box: w(full).grow(1).minW(0).minH(0).h(full).overflow(hidden) }
  style mainWorkbench = { box: display(flex).flexDir(column).basis(0).minH(0).minW(0).h(fill).overflow(hidden) }
</scope>

<row nowrap align="stretch" gap="0" style(shell)>
  <main testid="fill-shell" style(mainWorkbench, { box: grow(1).minW(0).w(full) })>
    <slot name="route"></slot>
  </main>
</row>
