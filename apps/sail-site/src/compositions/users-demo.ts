// users-demo — the data FIXTURE behind /sail/language/data's three-states
// demo. Lives under src/compositions so controllers may value-import it
// (sail-controller-import-convention). Was an inline `new Promise(...)`
// closure; extracted into an ES-imported client so `query()` can derive
// its cache key from the member path. A 900ms delay keeps loading→ok
// observable; `fail` drives the "fail the next one" button through error.

export const usersDemo = {
  users: {
    // `attempt` is carried in the args so each reload mints a fresh key
    // (a genuine re-fetch); `fail` reproduces the sabotage toggle.
    list: (args: { attempt: number; fail?: boolean }): Promise<string[]> =>
      new Promise((resolve, reject) => {
        setTimeout(() => {
          if (args.fail) reject(new Error('503 — the demo API was told to fail'))
          else resolve(['Ada', 'Grace', 'Edsger', 'Barbara'])
        }, 900)
      }),
  },
}

/** Docs/snippet alias — same client under the conventional teaching name. */
export const api = usersDemo
