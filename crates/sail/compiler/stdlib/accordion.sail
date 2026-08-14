<!--
  <accordion> — single disclosure. use:detailsToggle + use:detailsDisabled.
  Shared `name` = exclusive multi-item (native details name). Omit name for
  independent open; set `multiple` when several may stay open together.
-->

<scope>
  props: {
    open = false
    disabled = false
    name = ''
    multiple = false
    onToggle: (detail: { open: boolean }) => void
  }

  // Shell + trigger from background / foreground / border.
  style host = {display(block).border({ color: border, width: 1 }).radius(lg).bg(background).type({ family: body })}
  // list(none) + after chevron: kill UA marker; single themed caret.
  style summary = {display(flex).pad({ y: 3, x: 4 }).fg(foreground).radius(lg).cursor(pointer).select(none).type({ size: sm, weight: semibold, list: none }), when: {hover: bg(hover-bg), after: margin({ s: auto }).size(4).content(empty).bg(muted-foreground)}, animate: { transition: play(colors, fast) }}
  style content = {pad({ x: 4, y: 3 })}
</scope>

<details
  use:detailsToggle
  use:detailsDisabled
  style(host, {compound: [
      {disabled: true, filter({ opacity: 50 }).cursor(not-allowed)},
    ]})
>
  <summary style(summary)><slot name="trigger"></slot></summary>
  <div>
    <div style(content)><slot></slot></div>
  </div>
</details>
