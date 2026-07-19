import { queryKeyed } from '@ship/sail-runtime'

type Ctx = {
  ship: { agents: { summaries: () => Promise<{ items: { id: string; name: string }[] }> } }
}

export default function setup(ctx: unknown) {
  const { ship } = ctx as Ctx
  return {
    agents: queryKeyed(
      () => ['ship', 'agents', 'summaries'],
      () => ship.agents.summaries(),
      { retry: false },
    ),
  }
}
