<scope>
  props: {
    kind: 'all' | 't' | 'b' | 'l' | 'r' | 'none' | 'sm' | 'md' | 'lg' | 'xl' | 'full' = 'all'
  }
  style all = { size({ w: 16, h: 16 }), bg(primary), radius({ all: md }) }
  style edgeT = { size({ w: 16, h: 16 }), bg(primary), radius({ t: lg }) }
  style edgeB = { size({ w: 16, h: 16 }), bg(primary), radius({ b: lg }) }
  style edgeL = { size({ w: 16, h: 16 }), bg(primary), radius({ l: lg }) }
  style edgeR = { size({ w: 16, h: 16 }), bg(primary), radius({ r: lg }) }
  style presetNone = { size({ w: 16, h: 16 }), bg(primary), radius(none) }
  style presetSm = { size({ w: 16, h: 16 }), bg(primary), radius(sm) }
  style presetMd = { size({ w: 16, h: 16 }), bg(primary), radius(md) }
  style presetLg = { size({ w: 16, h: 16 }), bg(primary), radius(lg) }
  style presetXl = { size({ w: 16, h: 16 }), bg(primary), radius(xl) }
  style presetFull = { size({ w: 16, h: 16 }), bg(primary), radius(full) }
</scope>
<column style({ match: { kind: { all: all, t: edgeT, b: edgeB, l: edgeL, r: edgeR, none: presetNone, sm: presetSm, md: presetMd, lg: presetLg, xl: presetXl, full: presetFull } } })>
  <slot></slot>
</column>
