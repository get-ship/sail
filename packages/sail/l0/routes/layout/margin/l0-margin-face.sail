<scope>
  props: {
    kind: 'first' | 'below' | 'bar' | '1' | '2' | '2_5' | '3' | '4' | '5' | '6' | '7' | '8' | 'x' | 'y' | 't' | 'r' | 'b' | 'l' = 'bar'
  }
  style first = { size({ w: 16, h: 8 }), bg(accent) }
  style below = { margin({ t: 4 }), size({ w: 16, h: 8 }), bg(primary) }
  style bar = { size({ w: 8, h: 8 }), bg(accent) }
  style m1 = { margin(1), size({ w: 8, h: 8 }), bg(primary) }
  style m2 = { margin(2), size({ w: 8, h: 8 }), bg(primary) }
  style m2_5 = { margin(2_5), size({ w: 8, h: 8 }), bg(primary) }
  style m3 = { margin(3), size({ w: 8, h: 8 }), bg(primary) }
  style m4 = { margin(4), size({ w: 8, h: 8 }), bg(primary) }
  style m5 = { margin(5), size({ w: 8, h: 8 }), bg(primary) }
  style m6 = { margin(6), size({ w: 8, h: 8 }), bg(primary) }
  style m7 = { margin(7), size({ w: 8, h: 8 }), bg(primary) }
  style m8 = { margin(8), size({ w: 8, h: 8 }), bg(primary) }
  style edgeX = { margin({ x: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style edgeY = { margin({ y: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style edgeT = { margin({ t: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style edgeR = { margin({ r: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style edgeB = { margin({ b: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style edgeL = { margin({ l: 4 }), size({ w: 8, h: 8 }), bg(primary) }
</scope>
<column style({ match: { kind: { first: first, below: below, bar: bar, '1': m1, '2': m2, '2_5': m2_5, '3': m3, '4': m4, '5': m5, '6': m6, '7': m7, '8': m8, x: edgeX, y: edgeY, t: edgeT, r: edgeR, b: edgeB, l: edgeL } } })></column>
