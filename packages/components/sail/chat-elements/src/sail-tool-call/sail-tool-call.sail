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

  style host = {
    box: place({ position: relative }).overflow({ all: hidden }),
    paint: group().border({ color: border, width: 1 }).radius(md).fill({ color: surface-1 }),
  }
  style statusBg = {
    when: {
      groupStateRunning: { paint: fill({ color: info }) },
      groupStateOk: { paint: fill({ color: success }) },
      groupStateError: { paint: fill({ color: destructive }) },
    },
  }
  style statusFg = {
    when: {
      groupStateRunning: { paint: ink({ color: info }) },
      groupStateOk: { paint: ink({ color: success }) },
      groupStateError: { paint: ink({ color: destructive }) },
    },
  }
  // Status rail only — accordion owns the caret.
  style rail = { box: place({ position: absolute, top: 0, bottom: 0, left: 0 }).size({ w: 1 }) }
  style dot = { paint: elevation(none) }
  style statusLabel = {
    box: space({ ml: auto }),
    text: type({ size: xs, tracking: wide, case: uppercase }),
  }
  // padS clears the absolute rail so the first glyph never sits under it.
  style trigger = { box: space({ pad-y: 2, pad-l: 4, pad-r: 3 }) }
  style nameText = {
    paint: ink({ color: foreground }),
    text: type({ family: mono, size: sm, weight: medium, wrap: truncate }),
  }
  style body = {
    box: space({ pad-y: 2, pad-l: 4, pad-r: 3 }),
    paint: border({ edge: t, color: border, width: 1 }),
    text: type({ family: mono, size: sm, leading: normal }),
  }
  style sectionLabel = {
    box: flow({ display: block }).space({ mb: 1 }),
    paint: ink({ color: muted-foreground }),
    text: type({ family: body, size: xs, tracking: wide, case: uppercase }),
  }
  style pre = { text: type({ wrap: pre-wrap }) }
  style resultBlock = { box: space({ mt: 2 }) }
</scope>

<div data-state={status()} style(host)>
  <span aria-hidden="true" style(rail, statusBg)></span>

  <accordion open={open()} style({ paint: group() })>
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
