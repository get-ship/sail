// Rendering-mode examples for the deck Rendering page.

export const ssrConfig = `// sail.config.jsonc
{
  "deck": {
    "render": "ssr"
  }
}`

export const buildCmd = `$ sail build --project .
  ▸ wrote a self-contained static dir`
