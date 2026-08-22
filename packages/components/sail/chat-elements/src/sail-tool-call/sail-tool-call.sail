<!--
  <sail-tool-call> — collapsible tool invocation record.
  Header = name + status; body = args (default slot) + result slot.
  Composes <accordion> for disclosure. Status paint via group-state when maps
  (data-state={status}); no tone channel. L1 <span dot pulse> owns running anim.
-->

<scope>
  props: {
    name = ''
    status: 'running' | 'ok' | 'error' = 'running'
    open = false
  }

  style host = { position(relative).overflow({ all: hidden }).group().border({ color: border, width: 1 }).radius(md).bg(surface) }
  style statusBg = { when: { groupStateRunning: bg(info), groupStateOk: bg(success), groupStateError: bg(danger) } }
  style statusFg = { when: { groupStateRunning: fg(info), groupStateOk: fg(success), groupStateError: fg(danger) } }
  // Status rail only — accordion owns the caret.
  style rail = { position({ scheme: absolute, t: 0, b: 0, l: 0 }).size({ w: 1 }) }
  style dot = { elevation(none) }
  style statusLabel = { margin({ l: auto }).type({ size: xs, tracking: wide, case: uppercase }) }
  // padS clears the absolute rail so the first glyph never sits under it.
  style trigger = { pad({ y: 2, l: 4, r: 3 }) }
  style nameText = { fg(foreground).type({ family: mono, size: sm, weight: medium, wrap: truncate }) }
  style body = { pad({ y: 2, l: 4, r: 3 }).border(default-t).type({ family: mono, size: sm, leading: normal }) }
  style sectionLabel = { display(block).margin({ b: 1 }).fg(muted-foreground).type({ family: body, size: xs, tracking: wide, case: uppercase }) }
  style pre = { type({ wrap: pre-wrap }) }
  style resultBlock = { margin({ t: 2 }) }
</scope>

<div data-state={status()} style(host)>
  <span aria-hidden="true" style(rail, statusBg)></span>

  <accordion open={open()} style({ group() })>
    <row slot="trigger" gap="2" align="center" style(trigger)>
      <span
        dot
        aria-hidden="true"
        pulse={status() === 'running'}
        style(dot, statusBg)
      ></span>
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
