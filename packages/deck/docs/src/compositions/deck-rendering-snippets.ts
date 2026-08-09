// Rendering-mode examples for the Rendering page. Plain data module — never
// scope-scanned by the compiler. The SSR config contains `{ ... }`, which would
// parse as interpolation inside a template, so it must live here.

export const ssrConfig = `// sail.config.jsonc
{
  "deck": {
    "render": "ssr"
  }
}`

export const buildCmd = `$ sail build --project .
  ▸ wrote a self-contained static dir`
