// Code snippets for the Deck overview page, kept OUT of the paired
// controller.ts on purpose: the SAIL compiler scans a view's controller for
// `signal(...)` to infer scope, and example code containing literal braces
// must not be scope-scanned.

export const quickstart = `my-app/
└─ routes/
   └─ page.sail        <h1>Hello, deck.</h1>

$ deck dev
  ▸ serving http://127.0.0.1:7878`

export const layers = `SAIL    :  deck   :  your app
React  :  Next   :  your app`
