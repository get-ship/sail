// Code snippets for the deck data-seam page.

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
export default controller((ctx) => {
  const visits = ctx.store.visits
  return { visits }
})`
