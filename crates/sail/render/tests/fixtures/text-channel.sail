<!-- RENDER-TEXT-1: text-channel ladder (size/weight/tracking/case) + mixed inline span -->
<column style({ box: gap(4).pad(4), paint: bg(background).fg(foreground) })>
  <span style({ text: fontSize(xs) })>xs Hamburgefontsiv</span>
  <span style({ text: fontSize(sm) })>sm Hamburgefontsiv</span>
  <span style({ text: fontSize(base) })>base Hamburgefontsiv</span>
  <span style({ text: fontSize(lg) })>lg Hamburgefontsiv</span>
  <span style({ text: fontSize(xl) })>xl Hamburgefontsiv</span>
  <span style({ text: fontSize(2xl) })>2xl Hamburgefontsiv</span>
  <span style({ text: fontSize(4xl) })>4xl Hamburgefontsiv</span>
  <span style({ text: fontSize(lg).fontWeight(bold).tracking(wide).textCase(uppercase) })>
    tracking wide bold
  </span>
  <h1>Hero <span style({ paint: fg(primary) })>accent</span> word</h1>
</column>
