<scope>
  props: {
    kind: 'color-primary' | 'color-accent' | 'color-muted' | 'color-surface' | 'mix-15' | 'mix-50' | 'mix-85' | 'image-primary-accent' | 'image-none' | 'clip-border' | 'clip-padding' | 'clip-content' | 'clip-text' | 'glow-primary' | 'glow-accent' | 'glow-none' | 'grad-primary' | 'grad-accent' | 'grad-surface' | 'grad-none' = 'color-primary'
  }
  style colorPrimary = { size({ w: 16, h: 8 }), radius(sm), bg({ color: primary }) }
  style colorAccent = { size({ w: 16, h: 8 }), radius(sm), bg({ color: accent }) }
  style colorMuted = { size({ w: 16, h: 8 }), radius(sm), bg({ color: muted }) }
  style colorSurface = { size({ w: 16, h: 8 }), radius(sm), bg({ color: surface }) }
  style mix15 = { size({ w: 16, h: 8 }), radius(sm), bg({ color: primary, mix: 15 }) }
  style mix50 = { size({ w: 16, h: 8 }), radius(sm), bg({ color: primary, mix: 50 }) }
  style mix85 = { size({ w: 16, h: 8 }), radius(sm), bg({ color: primary, mix: 85 }) }
  style imagePrimaryAccent = { size({ w: 16, h: 8 }), radius(sm), bg({ image: primary-accent }) }
  style imageNone = { size({ w: 16, h: 8 }), radius(sm), bg({ color: muted, image: none }) }
  style clipBorder = { size({ w: 16, h: 8 }), pad(3), radius(sm), border({ width: 4, style: solid, color: accent }), bg({ color: primary, clip: border-box }) }
  style clipPadding = { size({ w: 16, h: 8 }), pad(3), radius(sm), border({ width: 4, style: solid, color: accent }), bg({ color: primary, clip: padding-box }) }
  style clipContent = { size({ w: 16, h: 8 }), pad(3), radius(sm), border({ width: 4, style: solid, color: accent }), bg({ color: primary, clip: content-box }) }
  style clipText = { size({ w: 16, h: 8 }), radius(sm), type(heading), bg({ gradient: primary, clip: text }), fg({ color: transparent }) }
  style glowPrimary = { size({ w: 16, h: 8 }), radius(sm), bg({ color: muted, glow: primary }) }
  style glowAccent = { size({ w: 16, h: 8 }), radius(sm), bg({ color: muted, glow: accent }) }
  style glowNone = { size({ w: 16, h: 8 }), radius(sm), bg({ color: muted, glow: none }) }
  style gradPrimary = { size({ w: 16, h: 8 }), radius(sm), bg({ gradient: primary }) }
  style gradAccent = { size({ w: 16, h: 8 }), radius(sm), bg({ gradient: accent }) }
  style gradSurface = { size({ w: 16, h: 8 }), radius(sm), bg({ color: muted, gradient: surface }) }
  style gradNone = { size({ w: 16, h: 8 }), radius(sm), bg({ color: muted, gradient: none }) }
</scope>
<column style({ match: { kind: { 'color-primary': colorPrimary, 'color-accent': colorAccent, 'color-muted': colorMuted, 'color-surface': colorSurface, 'mix-15': mix15, 'mix-50': mix50, 'mix-85': mix85, 'image-primary-accent': imagePrimaryAccent, 'image-none': imageNone, 'clip-border': clipBorder, 'clip-padding': clipPadding, 'clip-content': clipContent, 'clip-text': clipText, 'glow-primary': glowPrimary, 'glow-accent': glowAccent, 'glow-none': glowNone, 'grad-primary': gradPrimary, 'grad-accent': gradAccent, 'grad-surface': gradSurface, 'grad-none': gradNone } } })>
  <slot></slot>
</column>
