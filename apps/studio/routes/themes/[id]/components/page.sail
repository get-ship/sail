<!--
  /themes/[id]/components — L1 list + radius override.
  h(full)+minH(0) under the route-slot flex host (bare grow collapses).
-->
<scope>
  style surfaceFill = { box: grow(1).minH(0).h(full).minW(0).overflow(hidden) }
</scope>
<theme-components-surface style(surfaceFill)></theme-components-surface>
