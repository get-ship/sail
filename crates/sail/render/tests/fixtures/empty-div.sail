<!-- RENDER-TEXT-1: empty decorative boxes (size + w/h) must paint in flex -->
<column style({ box: pad(4).gap(4), paint: bg(background) })>
  <row style({ box: gap(3) })>
    <div style({ box: size(2), paint: bg(warning).radius(full) })></div>
    <div style({ box: size(4), paint: bg(primary).radius(sm) })></div>
    <div style({ box: w(16).h(8), paint: bg(accent).radius(md) })></div>
    <div style({ box: w(8).h(16), paint: bg(success).borderWidth(1).border(border) })></div>
  </row>
  <row style({ box: gap(2) })>
    <div style({ box: size(8), paint: bg(muted).radius(full) })></div>
    <div style({ box: size(12), paint: bg(destructive).opacity(80) })></div>
  </row>
</column>
