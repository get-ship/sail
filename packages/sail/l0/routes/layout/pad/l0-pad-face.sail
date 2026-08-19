<scope>
  props: {
    kind: 'pin' | '1' | '2' | '2_5' | '3' | '4' | '5' | '6' | '7' | '8' | 'x' | 'y' | 't' | 'r' | 'b' | 'l' = 'pin'
  }
  style inner = { size({ w: 4, h: 4 }), bg(muted) }
  style pin = { pad({ x: 4 }), size({ w: 16, h: 8 }), bg(primary) }
  style s1 = { pad(1), size({ w: 24, h: 24 }), bg(primary) }
  style s2 = { pad(2), size({ w: 24, h: 24 }), bg(primary) }
  style s2_5 = { pad(2_5), size({ w: 24, h: 24 }), bg(primary) }
  style s3 = { pad(3), size({ w: 24, h: 24 }), bg(primary) }
  style s4 = { pad(4), size({ w: 24, h: 24 }), bg(primary) }
  style s5 = { pad(5), size({ w: 24, h: 24 }), bg(primary) }
  style s6 = { pad(6), size({ w: 24, h: 24 }), bg(primary) }
  style s7 = { pad(7), size({ w: 24, h: 24 }), bg(primary) }
  style s8 = { pad(8), size({ w: 24, h: 24 }), bg(primary) }
  style edgeX = { pad({ x: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style edgeY = { pad({ y: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style edgeT = { pad({ t: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style edgeR = { pad({ r: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style edgeB = { pad({ b: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style edgeL = { pad({ l: 4 }), size({ w: 16, h: 8 }), bg(accent) }
</scope>
<column style({ match: { kind: { pin: pin, '1': s1, '2': s2, '2_5': s2_5, '3': s3, '4': s4, '5': s5, '6': s6, '7': s7, '8': s8, x: edgeX, y: edgeY, t: edgeT, r: edgeR, b: edgeB, l: edgeL } } })>
  <slot></slot>
  <if {kind() !== 'pin' && kind() !== 'x' && kind() !== 'y' && kind() !== 't' && kind() !== 'r' && kind() !== 'b' && kind() !== 'l'}>
    <column style(inner)></column>
  </if>
</column>
