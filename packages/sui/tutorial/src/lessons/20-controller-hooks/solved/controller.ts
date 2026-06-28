import { controller, signal, useFilteredList, useSelection } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

interface Agent { id: string; label: string }

const AGENTS: Agent[] = [
  { id: 'planner',  label: 'Planner — long-horizon work breakdown' },
  { id: 'reviewer', label: 'Reviewer — second-opinion code review' },
  { id: 'qa',       label: 'QA — Playwright + screenshot suite' },
  { id: 'docs',     label: 'Docs — Astro + Starlight site' },
  { id: 'release',  label: 'Release — changelog + version bump' },
]

export default controller<Scope>({
  scope: () => {
    const items = signal(AGENTS)
    return { items }
  },
  use: {
    list: useFilteredList(
      () => AGENTS,
      (a, q) => a.label.toLowerCase().includes(q.toLowerCase()),
    ),
    sel: useSelection(() => AGENTS, (a) => a.id),
  },
})
