<scope>
  props: {
    kind: 'hidden' | 'auto' | 'clip' | 'scroll' | 'visible' | 'clamp' | 'tall' | 'huge' = 'hidden'
  }
  style clip = { size({ w: 16, h: 16 }), overflow(hidden), bg(muted) }
  style auto = { size({ w: 16, h: 16 }), overflow(auto), bg(muted) }
  style clipWell = { size({ w: 16, h: 16 }), overflow(clip), bg(muted) }
  style scroll = { size({ w: 16, h: 16 }), overflow(scroll), bg(muted) }
  style vis = { size({ w: 16, h: 16 }), overflow(visible), bg(muted) }
  style clamp = { size({ max-w: 16, h: 8 }), overflow(hidden), bg(muted) }
  style tall = { size({ w: 8, h: 24 }), bg(accent) }
  style huge = { size({ w: 48, h: 8 }), bg(accent) }
</scope>
<column style({ match: { kind: { hidden: clip, auto: auto, clip: clipWell, scroll: scroll, visible: vis, clamp: clamp, tall: tall, huge: huge } } })>
  <slot></slot>
</column>
