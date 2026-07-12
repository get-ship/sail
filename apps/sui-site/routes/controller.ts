// Landing — the long-scroll pillar page. Every section owns a working
// centerpiece: the motion section deals, the theme section repaints the whole
// site, the data section filters and refetches, deck deploys, AI compiles.

import { controller, signal, effect, live, filteredList, type ViewScope } from '@ship/sui-runtime'
import { applyTheme, persistTheme, type ThemeId } from '@ship/tokens/themes'
import { targetCursor } from '@ship/sui-fx'
import type { Scope } from './page.sui'

const WORDS = ['Batteries', 'Components', 'Styling', 'State', 'Motion', 'Themes']

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const i = signal(0)
    effect(() => {
      const t = setInterval(() => i.set((n) => (n + 1) % WORDS.length), 2600)
      return () => clearInterval(t)
    })

    // MOTION — the deal.
    const dealt = signal(true)
    const replay = (): void => {
      dealt.set(false)
      setTimeout(() => dealt.set(true), 420)
    }

    // THEMES — repaint the world, inline.
    const theme = signal<string>('ship-light')
    const pick = (id: ThemeId): void => {
      applyTheme(id)
      persistTheme(id)
      theme.set(id)
    }

    // DATA — filter + refetch.
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
    const attempt = signal(1)
    const users = live(() => {
      attempt()
      return new Promise<string[]>((resolve) => setTimeout(() => resolve(['Ada', 'Grace', 'Edsger', 'Barbara']), 800))
    })
    const reload = (): void => attempt.set((a) => a + 1)

    // DECK — the deploy walk.
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
      targetCursor,
      dealt, replay,
      theme, pick,
      people, query, filtered, invite, users, reload,
      status, buildNo, deploy,
    }
  },
})
