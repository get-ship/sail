<!-- RENDER-TEXT-1: text-channel ladder (size/weight/tracking/case) + mixed inline span -->
<column gap="4" style({ pad(4).bg(background).fg(foreground) })>
  <span style({ type({ size: xs }) })>xs Hamburgefontsiv</span>
  <span style({ type({ size: sm }) })>sm Hamburgefontsiv</span>
  <span style({ type({ size: base }) })>base Hamburgefontsiv</span>
  <span style({ type({ size: lg }) })>lg Hamburgefontsiv</span>
  <span style({ type({ size: xl }) })>xl Hamburgefontsiv</span>
  <span style({ type({ size: 2xl }) })>2xl Hamburgefontsiv</span>
  <span style({ type({ size: 4xl }) })>4xl Hamburgefontsiv</span>
  <span style({ type({ size: lg, weight: bold, tracking: wide, case: uppercase }) })>
    tracking wide bold
  </span>
  <h1>Hero <span style({ fg(primary) })>accent</span> word</h1>
</column>
