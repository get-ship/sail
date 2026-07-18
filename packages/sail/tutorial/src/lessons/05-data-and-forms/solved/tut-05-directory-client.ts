// The "server" for lesson 05, moved OUT of the controller into an imported
// client module. query() derives its cache key from a static member-path
// call on an imported binding (`directory.team.list`), so the data source
// has to live behind such a binding — a local function in the controller
// gives the compiler no path to key on.
//
// `list` takes the tracked signals as call args: `attempt` re-keys each
// reload (a fresh cache entry → a real refetch), `broken` reproduces the
// "break the API" toggle. It returns a Promise, so query() routes it down
// the PULL path.

export interface Member {
  id: number
  name: string
  role: 'member' | 'admin'
}

let nextId = 4
const DB: Member[] = [
  { id: 1, name: 'Ada Lovelace', role: 'admin' },
  { id: 2, name: 'Grace Hopper', role: 'member' },
  { id: 3, name: 'Mary Shelley', role: 'member' },
]

export const directory = {
  team: {
    list: (_args: { attempt: number; broken: boolean }): Promise<Member[]> =>
      new Promise((resolve, reject) =>
        setTimeout(() => {
          if (_args.broken) {
            reject(new Error('503 — the team service is down. Uncheck "break the API" and reload.'))
          } else {
            resolve([...DB])
          }
        }, 600),
      ),
    add: (m: { name: string; role: 'member' | 'admin' }): void => {
      DB.push({ id: nextId++, ...m })
    },
  },
}
