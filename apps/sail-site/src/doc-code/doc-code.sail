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

  style host = { size(max) }
  style copyBtn = { fg(muted).radius(sm).elevation(xs) }
</scope>

<controller>
export default controller(({ code }: { code: () => string }) => {
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

<column gap="1" style(host)>
  <if {copy()}>
    <row justify="end">
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
    </row>
  </if>
  <code-block code={code()} lang={lang()}></code-block>
</column>
