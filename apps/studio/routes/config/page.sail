<!--
  /config — compiler, deck, project, studio prefs, plugins.
  size h/full + min-h 0 under the route-slot flex host (bare grow collapses).
-->
<scope>
  style surfaceFill = { box: flow({ grow: 1 }).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }) }
</scope>
<config-surface style(surfaceFill)></config-surface>
