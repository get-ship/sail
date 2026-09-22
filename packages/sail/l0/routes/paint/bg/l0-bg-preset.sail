<scope>
  props: {
    kind: 'app' | 'surface' | 'subtle' | 'inset' | 'overlay' | 'primary' | 'accent' | 'muted' = 'app'
  }
  style app = { size({ w: 24, h: 16 }), radius(sm), border({ width: 1, style: solid, color: muted }), bg(app) }
  style surface = { size({ w: 24, h: 16 }), radius(sm), border({ width: 1, style: solid, color: muted }), bg(surface) }
  style subtle = { size({ w: 24, h: 16 }), radius(sm), border({ width: 1, style: solid, color: muted }), bg(subtle) }
  style inset = { size({ w: 24, h: 16 }), radius(sm), border({ width: 1, style: solid, color: muted }), bg(inset) }
  style overlay = { size({ w: 24, h: 16 }), radius(sm), border({ width: 1, style: solid, color: muted }), bg(overlay) }
  style primary = { size({ w: 24, h: 16 }), radius(sm), border({ width: 1, style: solid, color: muted }), bg(primary) }
  style accent = { size({ w: 24, h: 16 }), radius(sm), border({ width: 1, style: solid, color: muted }), bg(accent) }
  style muted = { size({ w: 24, h: 16 }), radius(sm), border({ width: 1, style: solid, color: muted }), bg(muted) }
</scope>
<column style({ match: { kind: { app: app, surface: surface, subtle: subtle, inset: inset, overlay: overlay, primary: primary, accent: accent, muted: muted } } })></column>
