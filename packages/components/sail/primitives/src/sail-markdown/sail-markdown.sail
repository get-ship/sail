<!--
  <sail-markdown> — value prop → prose HTML via paintMarkdown action.
  No tone/size; prose children come from the parser (bindHtml), not SAIL tags.
-->

<scope>
  props: {
    value = ''
  }
</scope>

<controller>

import { bindHtml } from '@ship/sail-runtime/internal'
import type { Action } from '@ship/sail-runtime'
import type { Scope } from './sail-markdown.sail'
import { renderMarkdown } from './markdown-render'

export { renderMarkdown }

export default controller((ctx: Scope) => {
  const paintMarkdown: Action<void> = (node) => {
    const stop = bindHtml(node, () => renderMarkdown(ctx.value() ?? ''))
    return { destroy: stop }
  }
  return { paintMarkdown }
})

</controller>
<prose>
  <div use:paintMarkdown style({ box: flow({ display: block }) })>
    <span style({ box: flow({ display: contents }), text: type({ size: base }) })></span>
  </div>
</prose>
