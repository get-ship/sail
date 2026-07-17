// crm-demo — a minimal, real ESM client fixture proving out query()/
// mutation()'s derived-identity model end to end (query-rename-slice-spec.md).
//
// Unlike `demo.ts` (the app's existing `<client name="…">`-injected
// clients, whose methods return `LiveSource<T>` for the SSE/mux push
// path), `query()`'s fetcher contract is Promise-returning — the
// derived-identity story assumes `crm.deals.list(args)` is a plain
// async call, not a subscribe. This fixture is that shape: a plain
// ES-imported binding (`crm`) whose members are one-shot HTTP calls
// against the conformance server's demo op surface, so the compiler's
// query-key derivation pass (crates/sail/compiler/src/check/query_keys.rs)
// has a real, non-synthetic import to resolve.

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
