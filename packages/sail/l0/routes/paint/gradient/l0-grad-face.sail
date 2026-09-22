<scope>
  props: {
    kind: 'rail' | 'bg-primary' | 'bg-accent' | 'bg-surface' | 'bg-none' | 'fg-primary' | 'fg-accent' | 'fg-surface' | 'fg-none' | 'bd-primary' | 'bd-accent' | 'bd-surface' | 'bd-none' = 'rail'
  }
  style rail = { pad(2) }
  style bgPrimary = { size({ w: 10, h: 8 }), radius(sm), bg({ gradient: primary }) }
  style bgAccent = { size({ w: 10, h: 8 }), radius(sm), bg({ gradient: accent }) }
  style bgSurface = { size({ w: 10, h: 8 }), radius(sm), bg({ color: muted, gradient: surface }) }
  style bgNone = { size({ w: 10, h: 8 }), radius(sm), bg({ color: muted, gradient: none }) }
  style fgPrimary = { size({ w: 10, h: 8 }), radius(sm), type(heading), fg({ gradient: primary }) }
  style fgAccent = { size({ w: 10, h: 8 }), radius(sm), type(heading), fg({ gradient: accent }) }
  style fgSurface = { size({ w: 10, h: 8 }), radius(sm), type(heading), fg({ color: muted, gradient: surface }) }
  style fgNone = { size({ w: 10, h: 8 }), radius(sm), type(heading), fg({ color: muted, gradient: none }) }
  style bdPrimary = { size({ w: 10, h: 8 }), radius(sm), bg(surface), border({ width: 4, style: solid, gradient: primary }) }
  style bdAccent = { size({ w: 10, h: 8 }), radius(sm), bg(surface), border({ width: 4, style: solid, gradient: accent }) }
  style bdSurface = { size({ w: 10, h: 8 }), radius(sm), bg(surface), border({ width: 4, style: solid, color: muted, gradient: surface }) }
  style bdNone = { size({ w: 10, h: 8 }), radius(sm), bg(surface), border({ width: 4, style: solid, color: muted, gradient: none }) }
</scope>
<column style({ match: { kind: { rail: rail, 'bg-primary': bgPrimary, 'bg-accent': bgAccent, 'bg-surface': bgSurface, 'bg-none': bgNone, 'fg-primary': fgPrimary, 'fg-accent': fgAccent, 'fg-surface': fgSurface, 'fg-none': fgNone, 'bd-primary': bdPrimary, 'bd-accent': bdAccent, 'bd-surface': bdSurface, 'bd-none': bdNone } } })>
  <slot></slot>
</column>
