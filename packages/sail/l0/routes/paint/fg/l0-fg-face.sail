<scope>
  props: {
    kind: 'color-primary' | 'color-accent' | 'color-muted' | 'color-surface' | 'caret-auto' | 'caret-primary' | 'caret-transparent' | 'caret-current' | 'caret-currentColor' | 'accent-auto' | 'accent-primary' | 'accent-transparent' | 'accent-current' | 'accent-currentColor' | 'grad-primary' | 'grad-accent' | 'grad-surface' | 'grad-none' = 'color-primary'
  }
  style colorPrimary = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ color: primary, gradient: none }) }
  style colorAccent = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ color: accent, gradient: none }) }
  style colorMuted = { pad({ x: 4, y: 3 }), radius(sm), bg(primary), type(heading), fg({ color: muted, gradient: none }) }
  style colorSurface = { pad({ x: 4, y: 3 }), radius(sm), bg(primary), type(heading), fg({ color: surface, gradient: none }) }
  style caretAuto = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ caret: auto, gradient: none }) }
  style caretPrimary = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ caret: primary, gradient: none }) }
  style caretTransparent = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ caret: transparent, gradient: none }) }
  style caretCurrent = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ caret: current, gradient: none }) }
  style caretCurrentColor = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ caret: currentColor, gradient: none }) }
  style accentAuto = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ accent: auto, gradient: none }) }
  style accentPrimary = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ accent: primary, gradient: none }) }
  style accentTransparent = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ accent: transparent, gradient: none }) }
  style accentCurrent = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ accent: current, gradient: none }) }
  style accentCurrentColor = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ accent: currentColor, gradient: none }) }
  style gradPrimary = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ gradient: primary }) }
  style gradAccent = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ gradient: accent }) }
  style gradSurface = { pad({ x: 4, y: 3 }), radius(sm), bg(primary), type(heading), fg({ color: muted, gradient: surface }) }
  style gradNone = { pad({ x: 4, y: 3 }), radius(sm), bg(muted), type(heading), fg({ color: muted, gradient: none }) }
</scope>
<column style({ match: { kind: { 'color-primary': colorPrimary, 'color-accent': colorAccent, 'color-muted': colorMuted, 'color-surface': colorSurface, 'caret-auto': caretAuto, 'caret-primary': caretPrimary, 'caret-transparent': caretTransparent, 'caret-current': caretCurrent, 'caret-currentColor': caretCurrentColor, 'accent-auto': accentAuto, 'accent-primary': accentPrimary, 'accent-transparent': accentTransparent, 'accent-current': accentCurrent, 'accent-currentColor': accentCurrentColor, 'grad-primary': gradPrimary, 'grad-accent': gradAccent, 'grad-surface': gradSurface, 'grad-none': gradNone } } })>
  <slot></slot>
</column>
