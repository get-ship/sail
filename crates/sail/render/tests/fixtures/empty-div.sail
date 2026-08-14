<!-- RENDER-TEXT-1: empty decorative boxes (size + w/h) must paint in flex -->
<column gap="4" style({ pad(4), bg(background) })>
  <row gap="3">
    <div style({ size({ w: 2, h: 2 }), bg(warning).radius(full) })></div>
    <div style({ size({ w: 4, h: 4 }), bg(primary).radius(sm) })></div>
    <div style({ size({ w: 16, h: 8 }), bg(accent).radius(md) })></div>
    <div style({ size({ w: 8, h: 16 }), bg(success).border({ color: border, width: 1 }) })></div>
  </row>
  <row gap="2">
    <div style({ size({ w: 8, h: 8 }), bg(muted).radius(full) })></div>
    <div style({ size({ w: 12, h: 12 }), bg(destructive).filter({ opacity: 80 }) })></div>
  </row>
</column>
