// crm-demo — a minimal, real ESM client fixture proving out query()/
// mutation()'s derived-identity model end to end
// (query-rename-slice-spec.md). A plain ES-imported binding (`crm`)
// whose members are one-shot async calls — the shape the compiler's
// key-derivation pass (crates/sail/compiler/src/check/query_keys.rs)
// resolves a cache key from: the import (`crm`) is the key root, the
// static member path (`.deals.list`/`.deals.update`) is the key
// segments, the call args are the tracked tail.
//
// In-memory + a visible delay (matching /data's query() demo) rather
// than a real network round-trip — this route needs no new server
// surface, and the delay makes loading -> ok observable in the
// browser probe.

export interface Deal {
  id: string
  stage: 'open' | 'closed'
  amount: number
}

const DELAY_MS = 400

let deals: Deal[] = [
  { id: 'd1', stage: 'open', amount: 4200 },
  { id: 'd2', stage: 'open', amount: 9100 },
  { id: 'd3', stage: 'closed', amount: 1500 },
]

export const crm = {
  deals: {
    list: (args: { stage: string }): Promise<{ items: Deal[] }> =>
      new Promise((resolve) => {
        setTimeout(() => {
          resolve({ items: deals.filter((d) => d.stage === args.stage) })
        }, DELAY_MS)
      }),
    update: (args: { id: string; stage: 'open' | 'closed' }): Promise<Deal> =>
      new Promise((resolve, reject) => {
        setTimeout(() => {
          const deal = deals.find((d) => d.id === args.id)
          if (!deal) {
            reject(new Error(`no deal with id ${args.id}`))
            return
          }
          deal.stage = args.stage
          deals = deals.map((d) => (d.id === deal.id ? { ...deal } : d))
          resolve({ ...deal })
        }, DELAY_MS)
      }),
  },
}
