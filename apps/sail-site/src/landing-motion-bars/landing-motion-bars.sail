<!--
  Landing MOTION bento — springing primary bars (play(eq)), not a spinner.
  testid landing-motion-bar is the first bar (animation.spec.ts).
-->
<scope>
  style bar = { size(1).radius(sm).bg(primary).origin(bottom), animate: { loop: play(slide-up) } }
  style barDim = { size(1).radius(sm).bg(accent).origin(bottom), animate: { loop: play(slide-up) } }
</scope>

<row gap="1" align="end" id="landing-motion-bars">
  <div testid="landing-motion-bar" style(barDim)></div>
  <div style(bar)></div>
  <div style(barDim)></div>
  <div style(bar)></div>
  <div style(bar)></div>
  <div style(barDim)></div>
</row>
