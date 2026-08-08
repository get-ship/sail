<!--
  <sail-tool-call> — a collapsible record of one tool invocation. The
  header shows the tool name + a status dot; the body (args + result)
  collapses behind it. Composes <accordion> for the disclosure
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

  const host = style({ box: position(relative).overflow(hidden), paint: group().borderWidth(1).border(border).radius(md).bg(card) })
  const statusBg = style({ when: { groupStateRunning: { paint: bg(info) }, groupStateOk: { paint: bg(success) }, groupStateError: { paint: bg(destructive) } } })
  const statusFg = style({ when: { groupStateRunning: { paint: fg(info) }, groupStateOk: { paint: fg(success) }, groupStateError: { paint: fg(destructive) } } })
  // Status rail only — collapsible owns the caret (one disclosure chrome).
  const rail = style({ box: position(absolute).left(0).top(0).bottom(0).w(1) })
  const dot = style({ paint: shadow(none) })
  const statusLabel = style({ box: ml(auto), text: fontSize(xs).textCase(uppercase).tracking(wide) })
  // padS clears the absolute rail so the first glyph never sits under it.
  const trigger = style({ box: padY(2).padS(4).padE(3) })
  const nameText = style({ paint: fg(foreground), text: fontFamily(mono).fontSize(sm).fontWeight(medium).truncate() })
  const body = style({ box: padY(2).padS(4).padE(3), paint: borderTWidth(1).borderT(border), text: fontFamily(mono).fontSize(sm).leading(normal) })
  const sectionLabel = style({ box: display(block).mb(1), paint: fg(muted-foreground), text: fontFamily(body).fontSize(xs).textCase(uppercase).tracking(wide) })
  const pre = style({ text: whitespace(pre-wrap) })
  const resultBlock = style({ box: mt(2) })
</scope>

<div data-state={status()} style(host)>
  <span aria-hidden="true" style(rail, statusBg)></span>

  <accordion open={open()} style({ paint: group() })>
    <row slot="trigger" gap="2" align="center" style(trigger)>
      <span
        dot
        aria-hidden="true"
        pulse={status() === 'running'} style(dot, statusBg)></span>
      <span style(nameText)>{name()}</span>
      <span style(statusLabel, statusFg)>{status()}</span>
    </row>
    <div style(body)>
      <span style(sectionLabel)>Arguments</span>
      <div style(pre)><slot></slot></div>
      <div style(resultBlock)>
        <span style(sectionLabel)>Result</span>
        <div style(pre)><slot name="result"></slot></div>
      </div>
    </div>
  </accordion>
</div>
