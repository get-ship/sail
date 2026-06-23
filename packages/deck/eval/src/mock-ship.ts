// Default ship SDK stub fed to behavioral runs. Tasks that need
// specific responses can read/write to `ship.__test__` from inside
// their behavioralTests block.

export interface MockShip {
  __test__: Record<string, unknown>
  users: {
    create: (input: { name: string }) => Promise<{ id: string; name: string }>
    list: () => Promise<{ items: Array<{ id: string; name: string }> }>
  }
  agents: {
    summaries: () => Promise<{ items: Array<{ id: string; name: string }> }>
  }
  features: {
    list: () => Promise<{ items: string[] }>
  }
}

let userSeq = 0

export function createMockShip(): MockShip {
  const state = {
    users: [] as Array<{ id: string; name: string }>,
    agents: [
      { id: 'a1', name: 'Phoenix' },
      { id: 'a2', name: 'Compass' },
    ],
    features: ['boring infra', 'server-rendered first', 'agent-writable'],
  }
  const test: Record<string, unknown> = {}
  const ship: MockShip = {
    __test__: test,
    users: {
      create: async ({ name }) => {
        userSeq += 1
        const u = { id: `u${userSeq}`, name }
        state.users.push(u)
        return u
      },
      list: async () => ({ items: state.users.slice() }),
    },
    agents: {
      summaries: async () => ({ items: state.agents.slice() }),
    },
    features: {
      list: async () => {
        if (test['failNext']) {
          test['failNext'] = false
          throw new Error('mock: features.list failed')
        }
        return { items: state.features.slice() }
      },
    },
  }
  return ship
}
