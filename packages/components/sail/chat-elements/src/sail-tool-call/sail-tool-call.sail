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
    box: position(relative).overflow(hidden),
    paint: group().borderWidth(1).border(border).radius(md).bg(surface-1),
  }
  style statusBg = {
    when: {
      groupStateRunning: { paint: bg(info) },
      groupStateOk: { paint: bg(success) },
      groupStateError: { paint: bg(destructive) },
    },
  }
  style statusFg = {
    when: {
      groupStateRunning: { paint: fg(info) },
      groupStateOk: { paint: fg(success) },
      groupStateError: { paint: fg(destructive) },
    },
  }
  // Status rail only — accordion owns the caret.
  style rail = { box: position(absolute).left(0).top(0).bottom(0).w(1) }
  style dot = { paint: shadow(none) }
  style statusLabel = {
    box: ml(auto),
    text: fontSize(xs).textCase(uppercase).tracking(wide),
  }
  // padS clears the absolute rail so the first glyph never sits under it.
  style trigger = { box: padY(2).padS(4).padE(3) }
  style nameText = {
    paint: fg(foreground),
    text: fontFamily(mono).fontSize(sm).fontWeight(medium).truncate(),
  }
  style body = {
    box: padY(2).padS(4).padE(3),
    paint: borderTWidth(1).borderT(border),
    text: fontFamily(mono).fontSize(sm).leading(normal),
  }
  style sectionLabel = {
    box: display(block).mb(1),
    paint: fg(muted-foreground),
    text: fontFamily(body).fontSize(xs).textCase(uppercase).tracking(wide),
  }
  style pre = { text: whitespace(pre-wrap) }
  style resultBlock = { box: mt(2) }
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
