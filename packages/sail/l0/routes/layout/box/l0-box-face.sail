<scope>
  props: {
    kind: 'pin' | 'w' | 'h' | 'min-w' | 'min-h' | 'max-w' | 'max-h' | 'square' | 'video' | 'auto' | 'zero' | 'clamp' | 'huge' = 'pin'
  }
  style pin = { size({ w: 7, h: 8 }), bg(accent) }
  style onlyW = { size({ w: 16, h: 8 }), bg(primary) }
  style onlyH = { size({ w: 8, h: 16 }), bg(primary) }
  style minW = { size({ min-w: 16, h: 8 }), bg(accent) }
  style minH = { size({ w: 8, min-h: 16 }), bg(accent) }
  style maxW = { size({ max-w: 16, h: 8 }), bg(muted) }
  style maxH = { size({ w: 16, max-h: 8 }), bg(muted) }
  style square = { size({ w: 16, aspect: square }), bg(primary) }
  style video = { size({ w: 16, aspect: video }), bg(accent) }
  style auto = { size({ w: 16, h: 8, aspect: auto }), bg(muted) }
  style zero = { size({ w: 0, h: 0 }), bg(primary) }
  style clamp = { size({ max-w: 16 }), overflow(hidden), bg(muted) }
  style huge = { size({ w: 48, h: 8 }), bg(accent) }
</scope>
<column style({ match: { kind: { pin: pin, w: onlyW, h: onlyH, 'min-w': minW, 'min-h': minH, 'max-w': maxW, 'max-h': maxH, square: square, video: video, auto: auto, zero: zero, clamp: clamp, huge: huge } } })>
  <slot></slot>
</column>
