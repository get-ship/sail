<!--
  <doc-code> — docs wrapper: code-block + in-frame Copy on hover only.
  Copy is absolute *inside* the bordered pre (same pattern as sail-component-viewer).
  code-block pre-wraps long lines — never force horizontal scroll.
-->

<scope>
  props: {
    code = ''
    lang = 'sail'
    copy = true
  }

  style host = {
    box: display(block).position(relative).w(full),
    paint: group(),
  }
  style copyBtn = {
    box: position(absolute).top(3).right(3).z(2),
    paint: bg(surface-0).fg(muted-foreground).radius(sm).opacity(0).pointerEvents(none).shadow(xs),
    when: {
      group-hover: { paint: opacity(100).pointerEvents(auto) },
    },
  }
</scope>

<controller>
import type { Scope } from './doc-code.sail'

export default controller(({ code }: Scope) => {
  const copied = signal(false)
  let timer: ReturnType<typeof setTimeout> | null = null
  const copyLabel = derived(() => (copied() ? 'Copied' : 'Copy'))
  const clipText = (): string => code() ?? ''
  const markCopied = (): void => {
    copied.set(true)
    if (timer) clearTimeout(timer)
    timer = setTimeout(() => copied.set(false), 1500)
  }
  return { copyLabel, clipText, markCopied }
})
</controller>

<div style(host)>
  <if {copy()}>
    <button
      size="sm"
      variant="ghost"
      tone="muted"
      type="button"
      use:clipboard={{ text: clipText, onCopy: markCopied }}
      style(copyBtn)
    >
      {copyLabel()}
    </button>
  </if>
  <code-block code={code()} lang={lang()}></code-block>
</div>
