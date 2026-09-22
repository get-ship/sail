<scope>
  props: {
    kind: 'color-primary' | 'color-accent' | 'color-muted' | 'color-surface' | 'caret-auto' | 'caret-primary' | 'caret-transparent' | 'caret-current' | 'caret-currentColor' | 'accent-auto' | 'accent-primary' | 'accent-transparent' | 'accent-current' | 'accent-currentColor' | 'grad-primary' | 'grad-accent' | 'grad-surface' | 'grad-none' = 'color-primary'
  }
  style colorPrimary = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ color: primary }) }
  style colorAccent = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ color: accent }) }
  style colorMuted = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: primary }), type(heading), fg({ color: muted }) }
  style colorSurface = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: primary }), type(heading), fg({ color: surface }) }
  style caretAuto = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ caret: auto }) }
  style caretPrimary = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ caret: primary }) }
  style caretTransparent = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ caret: transparent }) }
  style caretCurrent = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ caret: current }) }
  style caretCurrentColor = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ caret: currentColor }) }
  style accentAuto = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ accent: auto }) }
  style accentPrimary = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ accent: primary }) }
  style accentTransparent = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ accent: transparent }) }
  style accentCurrent = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ accent: current }) }
  style accentCurrentColor = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ accent: currentColor }) }
  style gradPrimary = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ gradient: primary }) }
  style gradAccent = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: muted }), type(heading), fg({ gradient: accent }) }
  style gradSurface = { pad({ x: 4, y: 3 }), radius(sm), bg({ color: primary }), type(heading), fg({ color: muted, gradient: surface }) }
  style gradNone = { pad({ x: 4, y: 3 }), radius(sm), type(heading), fg({ color: muted, gradient: none }), bg({ color: primary }) }
</scope>
<column style({ match: { kind: { 'color-primary': colorPrimary, 'color-accent': colorAccent, 'color-muted': colorMuted, 'color-surface': colorSurface, 'caret-auto': caretAuto, 'caret-primary': caretPrimary, 'caret-transparent': caretTransparent, 'caret-current': caretCurrent, 'caret-currentColor': caretCurrentColor, 'accent-auto': accentAuto, 'accent-primary': accentPrimary, 'accent-transparent': accentTransparent, 'accent-current': accentCurrent, 'accent-currentColor': accentCurrentColor, 'grad-primary': gradPrimary, 'grad-accent': gradAccent, 'grad-surface': gradSurface, 'grad-none': gradNone } } })>
  <slot></slot>
</column>
