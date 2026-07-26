<!--
  <sail-tool-call> — a collapsible record of one tool invocation. The
  header shows the tool name + a status dot; the body (args + result)
  collapses behind it. Composes <collapsible> for the disclosure
  behavior (click / Enter / Space, animated height) rather than
  reimplementing it.

  Args arrive via the default slot; the result via the `result` slot.
  Both render monospace. Pure framework primitive — no SDK, no runtime.

  Status paint is Style match/when on semantic tokens (info / success /
  destructive) — no tone channel. The host stamps data-state={status} so
  rail, dot, and status text share one group-state-* when map. The L1
  `<span dot>` pulse attr (running only) owns the animation.

  Props
  ─────────────────────────────────────────────────────────────────────
  name    (string)  Tool name shown in the header.
  status  (union)   running | ok | error — colors the status chrome.
  open    (bool)    Whether the body starts expanded.
  ─────────────────────────────────────────────────────────────────────
  Slots
    (default)  tool arguments (monospace)
    result     tool result (monospace)
  ─────────────────────────────────────────────────────────────────────
-->

<scope>
  props: {
    name = ''
    status: 'running' | 'ok' | 'error' = 'running'
    open = false
  }

  const host = style.group().position(relative).borderWidth(1).border(border).radius(md).bg(card).overflow(hidden)
  const statusBg = when({
    groupStateRunning: style.bg(info),
    groupStateOk: style.bg(success),
    groupStateError: style.bg(destructive),
  })
  const statusFg = when({
    groupStateRunning: style.fg(info),
    groupStateOk: style.fg(success),
    groupStateError: style.fg(destructive),
  })
  const rail = style.position(absolute).left(0).top(0).bottom(0).w(0_75)
  const dot = style.shadow(none)
  const statusLabel = style.ml(auto).fontSize(xs).textCase(uppercase).tracking(wide)
  const trigger = style.padY(2).padX(3)
  const nameText = style.fontFamily(mono).fontSize(sm).fontWeight(medium).fg(foreground).truncate()
  const chevron = style.fg(muted-foreground).when({ groupOpen: style.rotate(90) })
  const body = style.borderTWidth(1).borderT(border).padY(2).padX(3).fontFamily(mono).fontSize(sm).leading(normal)
  const sectionLabel = style.display(block).fontFamily(body).fontSize(xs).textCase(uppercase).tracking(wide).fg(muted-foreground).mb(1)
  const pre = style.whitespace(pre-wrap)
  const resultBlock = style.mt(2)
</scope>

<div data-state={status()} sx={host}>
  <span aria-hidden="true" sx={[rail, statusBg]}></span>

  <collapsible open={open()} sx={style.group()}>
    <row slot="trigger" gap="2" align="center" sx={trigger}>
      <span
        dot
        aria-hidden="true"
        pulse={status() === 'running'}
        sx={[dot, statusBg]}
      ></span>
      <span sx={nameText}>{name()}</span>
      <span sx={[statusLabel, statusFg]}>{status()}</span>
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none"
        stroke="currentColor" stroke-width="1.5" stroke-linecap="round"
        stroke-linejoin="round" aria-hidden="true"
        grow="0" sx={chevron}>
        <path d="M4 2l4 4-4 4" />
      </svg>
    </row>
    <div sx={body}>
      <span sx={sectionLabel}>Arguments</span>
      <div sx={pre}><slot></slot></div>
      <div sx={resultBlock}>
        <span sx={sectionLabel}>Result</span>
        <div sx={pre}><slot name="result"></slot></div>
      </div>
    </div>
  </collapsible>
</div>
