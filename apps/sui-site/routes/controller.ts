// Landing: every panel below the fold WORKS — the headline rotates its
// battery, the state cell filters and grows a real list, the data cell walks
// loading/ok, the deploy cell walks live→rolling→live, and the themes button
// opens the site's actual picker.

import { controller, signal, effect, live, filteredList, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const WORDS = ['Batteries', 'Components', 'Styling', 'State', 'Motion', 'Themes']

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const i = signal(0)
    effect(() => {
      const t = setInterval(() => i.set((n) => (n + 1) % WORDS.length), 2800)
      return () => clearInterval(t)
    })

    const openThemes = (): void => {
      const panel = document.getElementById('sui-theme-panel') as HTMLElement & { showPopover?: () => void }
      try {
        panel?.showPopover?.()
      } catch {
        /* already open */
      }
    }

    // State cell — a real list behind the compositions.
    const people = signal([
      { id: 1, name: 'Ada Lovelace', status: 'active' },
      { id: 2, name: 'Grace Hopper', status: 'away' },
      { id: 3, name: 'Alan Turing', status: 'active' },
    ])
    const { query, filtered } = filteredList(() => people(), (p, q) => p.name.toLowerCase().includes(q))
    const invite = (): void => {
      const bench = ['Katherine Johnson', 'Annie Easley', 'Margaret Hamilton', 'Radia Perlman']
      people.set((l) => [...l, { id: l.length + 1, name: bench[(l.length - 3) % bench.length] ?? `Member ${l.length + 1}`, status: 'active' }])
    }

    // Data cell — a fake API with a visible delay; reload re-runs the source.
    const attempt = signal(1)
    const users = live(() => {
      attempt()
      return new Promise<string[]>((resolve) => setTimeout(() => resolve(['Ada', 'Grace', 'Edsger']), 800))
    })
    const reload = (): void => attempt.set((a) => a + 1)

    // Deploy cell — server-ish state walking through its phases.
    const status = signal<'live' | 'rolling'>('live')
    const buildNo = signal(214)
    const deploy = (): void => {
      status.set('rolling')
      setTimeout(() => {
        status.set('live')
        buildNo.set((n) => n + 1)
      }, 1400)
    }

    return {
      word: () => [WORDS[i()]],
      openThemes,
      people, query, filtered, invite,
      users, reload,
      status, buildNo, deploy,
    }
  },
})
