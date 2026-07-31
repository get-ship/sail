// Code snippets for the Deck overview page, kept OUT of the paired
// controller.ts on purpose: the Sail compiler scans a route's controller for
// `signal(...)` to infer scope, and example code containing literal braces
// must not be scope-scanned.

export const quickstart = `my-app/
└─ routes/
   └─ page.sail        <h1>Hello, deck.</h1>

$ sail dev
  ▸ serving http://127.0.0.1:7878`

export const layers = `Sail    :  deck   :  your app
React  :  Next   :  your app`
