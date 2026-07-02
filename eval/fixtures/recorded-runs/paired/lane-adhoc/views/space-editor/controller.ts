// Space Chrome editor controller — recorded lane output (hand-rolled shape).
export default {
  setup({ signal }: { signal: <T>(v: T) => ((() => T) & { set: (v: T) => void }) }) {
    const config = signal<Record<string, string>>({
      theme: 'paper',
      nav: 'top',
      dock: 'off',
      density: 'cozy',
      glyph: 'P',
      accent: 'indigo',
    })
    const editCount = signal(0)
    const dirty = signal(false)
    const spaceName = signal('atelier')

    return {
      config,
      editCount,
      dirty,
      spaceName,
      groups: () => [
        { id: 'theme', label: 'Theme', options: ['ship-light', 'paper', 'ship-dark', 'halo'] },
        { id: 'nav', label: 'Nav', options: ['rail', 'top', 'none'] },
        { id: 'dock', label: 'Dock', options: ['left', 'center', 'right', 'off'] },
        { id: 'density', label: 'Density', options: ['tight', 'cozy', 'loose'] },
      ],
      choose(id: string, option: string) {
        config.set({ ...config(), [id]: option })
        editCount.set(editCount() + 1)
        dirty.set(true)
      },
      save() {
        dirty.set(false)
      },
    }
  },
}
