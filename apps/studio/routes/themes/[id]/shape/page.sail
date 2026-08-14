<!--
  /themes/[id]/shape — character facet: radius, elevation, border, glass.
  size h/full + min-h 0 under the route-slot flex host (bare grow collapses).
-->
<scope>
  style surfaceFill = { grow(1).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
</scope>
<theme-shape-surface style(surfaceFill)></theme-shape-surface>
