// Lesson 05 mock data client. query() keys on the static member path
// `directory.team.list` — a local controller function has no key path.
// `list` args (`attempt`, `broken`) are tracked deps for refetch / fail.

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
