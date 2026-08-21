<scope>
  props: {
    kind: 'color-primary' | 'color-accent' | 'color-muted' | 'color-surface' | 'color-transparent' | 'color-current' | 'color-currentColor' | 'color-inherit' | 'color-none' | 'color-CanvasText' | 'color-Highlight' | 'mix-5' | 'mix-10' | 'mix-15' | 'mix-20' | 'mix-25' | 'mix-30' | 'mix-35' | 'mix-40' | 'mix-45' | 'mix-50' | 'mix-55' | 'mix-60' | 'mix-65' | 'mix-70' | 'mix-75' | 'mix-80' | 'mix-85' | 'mix-90' | 'mix-95' | 'image-primary-accent' | 'image-none' | 'clip-border' | 'clip-padding' | 'clip-content' | 'clip-text' | 'glow-primary' | 'glow-accent' | 'glow-none' | 'grad-primary' | 'grad-accent' | 'grad-surface' | 'grad-none' = 'color-primary'
  }
  style colorPrimary = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary }) }
  style colorAccent = { size({ w: 24, h: 16 }), radius(sm), bg({ color: accent }) }
  style colorMuted = { size({ w: 24, h: 16 }), radius(sm), bg({ color: muted }) }
  style colorSurface = { size({ w: 24, h: 16 }), radius(sm), bg({ color: surface }) }
  style colorTransparent = { size({ w: 24, h: 16 }), radius(sm), bg({ color: transparent }) }
  style colorCurrent = { size({ w: 24, h: 16 }), radius(sm), fg(primary), bg({ color: current }) }
  style colorCurrentColor = { size({ w: 24, h: 16 }), radius(sm), fg(primary), bg({ color: currentColor }) }
  style colorInherit = { size({ w: 24, h: 16 }), radius(sm), bg({ color: inherit }) }
  style colorNone = { size({ w: 24, h: 16 }), radius(sm), bg({ color: none }) }
  style colorCanvasText = { size({ w: 24, h: 16 }), radius(sm), bg({ color: CanvasText }) }
  style colorHighlight = { size({ w: 24, h: 16 }), radius(sm), bg({ color: Highlight }) }
  style mix5 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 5 }) }
  style mix10 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 10 }) }
  style mix15 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 15 }) }
  style mix20 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 20 }) }
  style mix25 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 25 }) }
  style mix30 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 30 }) }
  style mix35 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 35 }) }
  style mix40 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 40 }) }
  style mix45 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 45 }) }
  style mix50 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 50 }) }
  style mix55 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 55 }) }
  style mix60 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 60 }) }
  style mix65 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 65 }) }
  style mix70 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 70 }) }
  style mix75 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 75 }) }
  style mix80 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 80 }) }
  style mix85 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 85 }) }
  style mix90 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 90 }) }
  style mix95 = { size({ w: 24, h: 16 }), radius(sm), bg({ color: primary, mix: 95 }) }
  style imagePrimaryAccent = { size({ w: 24, h: 16 }), radius(sm), bg({ image: primary-accent }) }
  style imageNone = { size({ w: 24, h: 16 }), radius(sm), bg({ color: muted, image: none }) }
  style clipBorder = { size({ w: 24, h: 16 }), pad(3), radius(sm), border({ width: 4, style: solid, color: accent }), bg({ color: primary, clip: border-box }) }
  style clipPadding = { size({ w: 24, h: 16 }), pad(3), radius(sm), border({ width: 4, style: solid, color: accent }), bg({ color: primary, clip: padding-box }) }
  style clipContent = { size({ w: 24, h: 16 }), pad(3), radius(sm), border({ width: 4, style: solid, color: accent }), bg({ color: primary, clip: content-box }) }
  style clipText = { size({ w: 24, h: 16 }), radius(sm), type(heading), bg({ gradient: primary, clip: text }), fg({ color: transparent }) }
  style glowPrimary = { size({ w: 24, h: 16 }), radius(sm), bg({ color: muted, glow: primary }) }
  style glowAccent = { size({ w: 24, h: 16 }), radius(sm), bg({ color: muted, glow: accent }) }
  style glowNone = { size({ w: 24, h: 16 }), radius(sm), bg({ color: muted, glow: none }) }
  style gradPrimary = { size({ w: 24, h: 16 }), radius(sm), bg({ gradient: primary }) }
  style gradAccent = { size({ w: 24, h: 16 }), radius(sm), bg({ gradient: accent }) }
  style gradSurface = { size({ w: 24, h: 16 }), radius(sm), bg({ color: muted, gradient: surface }) }
  style gradNone = { size({ w: 24, h: 16 }), radius(sm), bg({ color: muted, gradient: none }) }
</scope>
<column style({ match: { kind: { 'color-primary': colorPrimary, 'color-accent': colorAccent, 'color-muted': colorMuted, 'color-surface': colorSurface, 'color-transparent': colorTransparent, 'color-current': colorCurrent, 'color-currentColor': colorCurrentColor, 'color-inherit': colorInherit, 'color-none': colorNone, 'color-CanvasText': colorCanvasText, 'color-Highlight': colorHighlight, 'mix-5': mix5, 'mix-10': mix10, 'mix-15': mix15, 'mix-20': mix20, 'mix-25': mix25, 'mix-30': mix30, 'mix-35': mix35, 'mix-40': mix40, 'mix-45': mix45, 'mix-50': mix50, 'mix-55': mix55, 'mix-60': mix60, 'mix-65': mix65, 'mix-70': mix70, 'mix-75': mix75, 'mix-80': mix80, 'mix-85': mix85, 'mix-90': mix90, 'mix-95': mix95, 'image-primary-accent': imagePrimaryAccent, 'image-none': imageNone, 'clip-border': clipBorder, 'clip-padding': clipPadding, 'clip-content': clipContent, 'clip-text': clipText, 'glow-primary': glowPrimary, 'glow-accent': glowAccent, 'glow-none': glowNone, 'grad-primary': gradPrimary, 'grad-accent': gradAccent, 'grad-surface': gradSurface, 'grad-none': gradNone } } })>
  <slot></slot>
</column>
