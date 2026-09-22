<scope>
  props: {
    kind: 'host' | 'pin' | 'static' | 'rel' | 'stick-well' | 'stick' | 'stick-pad' | 'fixed' | 'pin-br' | 'inset-0' | 'inset-auto' | 'edges-auto' = 'host'
  }
  style host = { position(relative), size({ w: 24, h: 16 }), bg(muted) }
  style pin = { position({ scheme: absolute, t: 4, l: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style stat = { position(static), size({ w: 16, h: 8 }), bg(accent) }
  style rel = { position(relative), size({ w: 16, h: 8 }), bg(primary) }
  style stickWell = { size({ w: 24, h: 16 }), overflow(auto), bg(muted) }
  style stick = { position({ scheme: sticky, t: 0 }), size({ w: 16, h: 8 }), bg(accent) }
  style stickPad = { size({ h: 24 }), bg(muted) }
  style fix = { position(fixed), size({ w: 16, h: 8 }), bg(accent) }
  style pinBr = { position({ scheme: absolute, b: 4, r: 4 }), size({ w: 8, h: 8 }), bg(accent) }
  style inset0 = { position({ scheme: absolute, inset: 0 }), bg(primary) }
  style insetAuto = { position({ scheme: absolute, inset: auto }), size({ w: 8, h: 8 }), bg(accent) }
  style edgesAuto = { position({ scheme: absolute, t: auto, b: auto, l: auto, r: auto }), size({ w: 8, h: 8 }), bg(primary) }
</scope>
<column style({ match: { kind: { host: host, pin: pin, static: stat, rel: rel, 'stick-well': stickWell, stick: stick, 'stick-pad': stickPad, fixed: fix, 'pin-br': pinBr, 'inset-0': inset0, 'inset-auto': insetAuto, 'edges-auto': edgesAuto } } })>
  <slot></slot>
</column>
