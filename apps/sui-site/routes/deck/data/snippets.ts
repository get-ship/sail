// Code snippets for the deck data-seam page.

export const configExample = `// deck.jsonc
{
  "entry": {
    "clientsModule": "./src/clients.ts",
    "exposeNames": ["store"]
  }
}`

export const usageExample = `// any controller
setup: (ctx) => {
  const visits = ctx.clients.store.visits
  return { visits }
}`
