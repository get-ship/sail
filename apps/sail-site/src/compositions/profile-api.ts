// profile-api — the fake server behind /docs/guide/forms' live demo. Lives
// under src/compositions so controllers may value-import it
// (sail-controller-import-convention). A 900ms delay keeps the pending state
// visible; `fail` lets the "Fail the next save" button walk the reader
// through the error path on demand.

export interface Profile {
  name: string
  email: string
}

export const profileApi = {
  save: (_values: Profile, fail: boolean): Promise<void> =>
    new Promise((resolve, reject) => {
      setTimeout(() => {
        if (fail) reject(new Error('503 — the demo API was told to fail'))
        else resolve()
      }, 900)
    }),
}
