<scope>
  props: {
    kind: 'host' | 'pin' | 'static' | 'rel' | 'stick-well' | 'stick' | 'stick-pad' = 'host'
  }
  style host = { position(relative), size({ w: 24, h: 16 }), bg(muted) }
  style pin = { position({ scheme: absolute, t: 4, l: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style stat = { position(static), size({ w: 16, h: 8 }), bg(accent) }
  style rel = { position(relative), size({ w: 16, h: 8 }), bg(primary) }
  style stickWell = { size({ w: 24, h: 16 }), overflow(auto), bg(muted) }
  style stick = { position({ scheme: sticky, t: 0 }), size({ w: 16, h: 8 }), bg(accent) }
  style stickPad = { size({ h: 24 }), bg(muted) }
</scope>
<column style({ match: { kind: { host: host, pin: pin, static: stat, rel: rel, 'stick-well': stickWell, stick: stick, 'stick-pad': stickPad } } })>
  <slot></slot>
</column>
