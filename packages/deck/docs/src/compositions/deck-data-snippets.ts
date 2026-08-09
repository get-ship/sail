// Code snippets for the data-seam page (kept out of controller.ts so braces /
// scope-like tokens don't leak into the inferred scope).

export const configExample = `// sail.config.jsonc
{
  "deck": {
    "entry": {
      "clientsModule": "./src/clients.ts",
      "exposeNames": ["store"]
    }
  }
}`

export const usageExample = `// any controller
setup: (ctx) => {
  const visits = ctx.clients.store.visits
  return { visits }
}`
