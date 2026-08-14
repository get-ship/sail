<!--
  /themes/[id]/assets — type + icons deep facet.
  size h/full + min-h 0 under the route-slot flex host (bare grow collapses).
-->
<scope>
  style surfaceFill = { grow(1).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
</scope>
<theme-assets-surface style(surfaceFill)></theme-assets-surface>
