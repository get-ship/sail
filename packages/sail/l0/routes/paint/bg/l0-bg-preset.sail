<scope>
  props: {
    kind: 'primary' | 'accent' | 'muted' | 'surface' = 'primary'
  }
  style primary = { size({ w: 16, h: 8 }), radius(sm), bg(primary) }
  style accent = { size({ w: 16, h: 8 }), radius(sm), bg(accent) }
  style muted = { size({ w: 16, h: 8 }), radius(sm), bg(muted) }
  style surface = { size({ w: 16, h: 8 }), radius(sm), bg(surface) }
</scope>
<column style({ match: { kind: { primary: primary, accent: accent, muted: muted, surface: surface } } })></column>
