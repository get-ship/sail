// Promise-shaped CRM fixture for query()/mutation() identity proofs.
// Members are one-shot HTTP against the conformance demo op surface
// (not LiveSource SSE like app/clients/demo.ts).

export interface Deal {
  id: string
  stage: 'open' | 'closed'
  amount: number
}

export const crm = {
  deals: {
    list: async (args: { stage: string }, signal?: AbortSignal): Promise<{ items: Deal[] }> => {
      const qs = new URLSearchParams({ stage: args.stage }).toString()
      const res = await fetch(`/_deck/api/demo/deals?${qs}`, { credentials: 'same-origin', signal })
      return res.json() as Promise<{ items: Deal[] }>
    },
    update: async (args: { id: string; stage: string }, signal?: AbortSignal): Promise<Deal> => {
      const res = await fetch('/_deck/api/demo/deals/update', {
        method: 'POST',
        credentials: 'same-origin',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(args),
        signal,
      })
      return res.json() as Promise<Deal>
    },
  },
}
