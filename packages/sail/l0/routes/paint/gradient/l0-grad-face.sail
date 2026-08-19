<scope>
  props: {
    kind: 'rail' | 'bg-primary' | 'bg-accent' | 'bg-surface' | 'bg-none' | 'fg-primary' | 'fg-accent' | 'fg-surface' | 'fg-none' | 'bd-primary' | 'bd-accent' | 'bd-surface' | 'bd-none' = 'rail'
  }
  style rail = { size({ w: 100%, h: 24 }) }
  style bgPrimary = { size(grow), radius(sm), bg({ gradient: primary }) }
  style bgAccent = { size(grow), radius(sm), bg({ gradient: accent }) }
  style bgSurface = { size(grow), radius(sm), bg({ color: muted, gradient: surface }) }
  style bgNone = { size(grow), radius(sm), bg({ color: muted, gradient: none }) }
  style fgPrimary = { size(grow), radius(sm), type(heading), fg({ gradient: primary }) }
  style fgAccent = { size(grow), radius(sm), type(heading), fg({ gradient: accent }) }
  style fgSurface = { size(grow), radius(sm), type(heading), fg({ color: muted, gradient: surface }) }
  style fgNone = { size(grow), radius(sm), type(heading), fg({ color: muted, gradient: none }) }
  style bdPrimary = { size(grow), radius(sm), bg(surface), border({ width: 4, style: solid, gradient: primary }) }
  style bdAccent = { size(grow), radius(sm), bg(surface), border({ width: 4, style: solid, gradient: accent }) }
  style bdSurface = { size(grow), radius(sm), bg(surface), border({ width: 4, style: solid, color: muted, gradient: surface }) }
  style bdNone = { size(grow), radius(sm), bg(surface), border({ width: 4, style: solid, color: muted, gradient: none }) }
</scope>
<column style({ match: { kind: { rail: rail, 'bg-primary': bgPrimary, 'bg-accent': bgAccent, 'bg-surface': bgSurface, 'bg-none': bgNone, 'fg-primary': fgPrimary, 'fg-accent': fgAccent, 'fg-surface': fgSurface, 'fg-none': fgNone, 'bd-primary': bdPrimary, 'bd-accent': bdAccent, 'bd-surface': bdSurface, 'bd-none': bdNone } } })>
  <slot></slot>
</column>
