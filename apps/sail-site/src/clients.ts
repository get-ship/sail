// Clients seam for the Sail docs site. Pages own their own scope; the shell
// owns theme + base styles. Opt in to the tree-sitter `.sail` painter so
// `<code-block>` matches VS Code / Studio. TS / shell stay on the runtime
// regex. Missing wasm keeps that regex — hello-world never registers.

import { registerSailHighlighter } from '@ship/sail-runtime/actions'

void import('@ship/sail-highlight')
  .then(async (hl) => {
    await hl.ensureHighlight()
    registerSailHighlighter(hl.sailTokHtml)
  })
  .catch(() => {
    /* regex fallback */
  })

