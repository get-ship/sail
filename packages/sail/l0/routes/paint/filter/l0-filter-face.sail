<scope>
  props: {
    kind: 'blur' | 'brightness' | 'contrast' | 'grayscale' | 'hue' | 'invert' | 'sepia' | 'saturate' | 'backdrop' | 'blend' | 'opacity' | 'under' = 'blur'
  }
  style blur = { size({ w: 24, h: 16 }), radius(sm), bg(primary), filter({ blur: 4 }) }
  style brightness = { size({ w: 24, h: 16 }), radius(sm), bg(primary), filter({ brightness: 150 }) }
  style contrast = { size({ w: 24, h: 16 }), radius(sm), bg(primary), filter({ contrast: 200 }) }
  style grayscale = { size({ w: 24, h: 16 }), radius(sm), bg(accent), filter({ grayscale: 100 }) }
  style hue = { size({ w: 24, h: 16 }), radius(sm), bg(primary), filter({ hue: 180 }) }
  style invert = { size({ w: 24, h: 16 }), radius(sm), bg(primary), filter({ invert: 100 }) }
  style sepia = { size({ w: 24, h: 16 }), radius(sm), bg(primary), filter({ sepia: 100 }) }
  style saturate = { size({ w: 24, h: 16 }), radius(sm), bg(accent), filter({ saturate: 200 }) }
  style backdrop = { size({ w: 24, h: 16 }), radius(sm), bg(transparent), filter({ backdrop: filter-backdrop }) }
  style blend = { size({ w: 24, h: 16 }), radius(sm), bg(accent), filter({ blend: multiply }) }
  style opacity = { size({ w: 24, h: 16 }), radius(sm), bg(primary), filter({ opacity: 50 }) }
  style under = { size({ w: 24, h: 16 }), radius(sm), bg(primary) }
</scope>
<column style({ match: { kind: { blur: blur, brightness: brightness, contrast: contrast, grayscale: grayscale, hue: hue, invert: invert, sepia: sepia, saturate: saturate, backdrop: backdrop, blend: blend, opacity: opacity, under: under } } })>
  <slot></slot>
</column>
