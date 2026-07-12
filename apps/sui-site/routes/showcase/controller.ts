// /showcase — screens that WORK, not mockups. The team screen runs the
// filteredList + selection compositions over a mutable list; the fleet
// screen walks a deploy through its states and jitters latency on a
// mount-owned interval so the dashboard reads as live.

import { controller, signal, effect, filteredList, selection, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { code } from './snippets'

interface Person {
  id: number
  name: string
  team: string
  status: 'active' | 'away'
}
interface Service {
  id: string
  name: string
  status: 'live' | 'rolling'
  p99: number
  load: number
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const people = signal<Person[]>([
      { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
      { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'active' },
      { id: 3, name: 'Alan Turing', team: 'Research', status: 'away' },
      { id: 4, name: 'Barbara Liskov', team: 'Compilers', status: 'active' },
      { id: 5, name: 'Edsger Dijkstra', team: 'Research', status: 'away' },
    ])
    const { query, filtered } = filteredList(() => people(), (p, q) =>
      (p.name + ' ' + p.team).toLowerCase().includes(q),
    )
    const { selected, toggle, clear } = selection(() => people(), (p) => p.id)

    const flipStatus = (): void => {
      const cur = selected()
      if (!cur) return
      people.set((list) =>
        list.map((p) =>
          p.id === cur.id ? { ...p, status: p.status === 'active' ? 'away' : 'active' } : p,
        ),
      )
    }
    const remove = (): void => {
      const cur = selected()
      if (!cur) return
      people.set((list) => list.filter((p) => p.id !== cur.id))
      clear()
    }

    const services = signal<Service[]>([
      { id: 'api', name: 'api-gateway', status: 'live', p99: 41, load: 82 },
      { id: 'search', name: 'search-index', status: 'live', p99: 118, load: 47 },
      { id: 'billing', name: 'billing-worker', status: 'live', p99: 22, load: 94 },
    ])
    const deploys = signal<string[]>(['#212 · billing-worker', '#213 · search-index'])
    const deploy = (id: string): void => {
      services.set((l) => l.map((s) => (s.id === id ? { ...s, status: 'rolling' } : s)))
      setTimeout(() => {
        services.set((l) => l.map((s) => (s.id === id ? { ...s, status: 'live' } : s)))
        const name = services().find((s) => s.id === id)?.name ?? id
        deploys.set((d) => [`#${214 + d.length - 2} · ${name}`, ...d].slice(0, 4))
      }, 1500)
    }

    // Ambient life: latency and load drift so the dashboard reads as live.
    // An effect with no tracked reads runs once; its returned cleanup runs on
    // unmount — the idiomatic home for an interval.
    effect(() => {
      const t = setInterval(() => {
        services.set((l) =>
          l.map((s) => ({
            ...s,
            p99: Math.max(8, Math.round(s.p99 + (Math.random() - 0.5) * 14)),
            load: Math.min(100, Math.max(5, Math.round(s.load + (Math.random() - 0.5) * 10))),
          })),
        )
      }, 1200)
      return () => clearInterval(t)
    })

    // The player. Position drifts on an effect interval that TRACKS playing():
    // pause re-runs the effect, whose cleanup stops the timer — the same idiom
    // that stops it on unmount. The equalizer is not simulated: the bars only
    // exist (and loop) while playing() holds them in the DOM.
    const playing = signal(true)
    const position = signal(23)
    effect(() => {
      if (!playing()) return
      const t = setInterval(() => position.set((p) => (p + 1) % 100), 150)
      return () => clearInterval(t)
    })
    const togglePlay = (): void => playing.set((p) => !p)

    return {
      code, people, query, filtered, selected, toggle, flipStatus, remove,
      services, deploys, deploy,
      playing, position, togglePlay,
    }
  },
})
