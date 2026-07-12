// /overview — the first page carries a WORKING demo, not a still life: a
// filterable team list where Invite actually adds a row. The snippet above
// it is this exact program in its single-file form.

import { controller, signal, filteredList, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { code } from './snippets'

const BENCH = ['Katherine Johnson', 'Annie Easley', 'Margaret Hamilton', 'Radia Perlman']

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const people = signal([
      { id: 1, name: 'Ada Lovelace', team: 'Engineering', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Engineering', status: 'away' },
      { id: 3, name: 'Alan Turing', team: 'Research', status: 'active' },
    ])
    const { query, filtered } = filteredList(() => people(), (p, q) =>
      (p.name + ' ' + p.team).toLowerCase().includes(q),
    )
    const invite = (): void => {
      people.set((l) => [
        ...l,
        { id: l.length + 1, name: BENCH[(l.length - 3) % BENCH.length] ?? `Member ${l.length + 1}`, team: 'New', status: 'active' },
      ])
    }
    return { code, query, filtered, invite }
  },
})
