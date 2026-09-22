import '@ship/sail-components/code'
import '@ship/sail-components/chat'
import '@ship/sail-components/primitives'
import '@ship/sail-components/viewers'
import '@ship/tokens/tokens.css'
import '@ship/tokens/themes.css'
import '@ship/tokens/sail-base.css'
import '@ship/tokens/element-skins.css'
import { registerSailHighlighter } from '@ship/sail-runtime/actions'

void import('@ship/sail-highlight')
  .then(async (hl) => {
    await hl.ensureHighlight()
    registerSailHighlighter(hl.sailTokHtml)
  })
  .catch(() => {
    /* regex fallback */
  })
