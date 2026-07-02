// Space Chrome editor controller — recorded lane output (composed shape).
// The chrome config is local editor state (a signal); the preview derives
// from it. Save calls the faked ship.spaces.saveConfig (no-op in tests).

interface SpaceChrome {
  theme: 'ship-light' | 'paper' | 'ship-dark' | 'halo'
  nav: 'rail' | 'top' | 'none'
  dock: 'left' | 'center' | 'right' | 'off'
  density: 'tight' | 'cozy' | 'loose'
  glyph: string
  accent: string
}

const DEFAULTS: SpaceChrome = {
  theme: 'paper',
  nav: 'top',
  dock: 'off',
  density: 'cozy',
  glyph: 'P',
  accent: 'indigo',
}

const GROUPS = [
  { id: 'theme', label: 'Theme', options: ['ship-light', 'paper', 'ship-dark', 'halo'] },
  { id: 'nav', label: 'Nav', options: ['rail', 'top', 'none'] },
  { id: 'dock', label: 'Dock', options: ['left', 'center', 'right', 'off'] },
  { id: 'density', label: 'Density', options: ['tight', 'cozy', 'loose'] },
  { id: 'glyph', label: 'Glyph', options: ['P', '◆', '✦', '▲', 'M'] },
  { id: 'accent', label: 'Accent', options: ['indigo', 'teal', 'amber', 'rose'] },
] as const

export default {
  setup({ signal }: { signal: <T>(v: T) => ((() => T) & { set: (v: T) => void }) }) {
    const config = signal<SpaceChrome>({ ...DEFAULTS })
    const editCount = signal(0)
    const dirty = signal(false)
    const spaceName = signal('atelier')

    return {
      config,
      editCount,
      dirty,
      spaceName,
      groups: () => GROUPS,
      choose(id: keyof SpaceChrome, option: string) {
        config.set({ ...config(), [id]: option })
        editCount.set(editCount() + 1)
        dirty.set(true)
      },
      save() {
        // faked SDK call — no-op in tests
        void Promise.resolve({ saved: config() })
        dirty.set(false)
      },
    }
  },
}
