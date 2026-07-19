// profile-api — the fake server behind /sail/guide/forms' live demo. A 900ms
// delay keeps the pending state visible; `fail` lets the "Fail the next save"
// button walk the reader through the error path on demand.

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
