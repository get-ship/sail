<!--
  Showcase now-playing EQ. Three looping bars — play(slide-up), primary.
  Overlay position is L1-only; bars sit in-flow on the current track.
-->
<scope>
  style bar = { size(1).radius(full).bg(primary).origin(bottom), animate: { loop: play(slide-up) } }
  style barMid = { size(1).radius(full).bg(primary).origin(bottom), animate: { loop: play(slide-up) } }
</scope>

<row gap="1" align="end">
  <i style(bar)></i>
  <i style(barMid)></i>
  <i style(bar)></i>
</row>
