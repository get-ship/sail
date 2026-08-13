<!--
  /themes/[id] — theme overview (editorial + docked specimen).
  size h/full + min-h 0 under the route-slot flex host (bare grow collapses).
-->
<scope>
  style surfaceFill = { box: flow({ grow: 1 }).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
</scope>
<theme-overview-surface style(surfaceFill)></theme-overview-surface>
