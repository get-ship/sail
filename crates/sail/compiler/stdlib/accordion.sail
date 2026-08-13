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

  // Shell + trigger from component.accordion tokens.
  style host = {box: flow({ display: block }), paint: border({ color: accordion.border, width: 1 }).radius(lg).fill({ color: accordion }), text: type({ family: body })}
  // list(none) + after chevron: kill UA marker; single themed caret.
  style summary = {box: flow({ display: flex }).space({ pad-y: 3, pad-x: 4 }), paint: ink({ color: accordion.foreground }).radius(lg).motion({ property: colors, duration: fast, ease: out }).cursor(pointer).select(none), text: type({ size: sm, weight: semibold, list: none }), when: {hover: {paint: fill({ color: hover-bg })}, after: {box: space({ ms: auto }).size(4), paint: content(empty).fill({ color: accordion.chevron })}}}
  style content = {box: space({ pad-x: 4, pad-y: 3 })}
</scope>

<details
  use:detailsToggle
  use:detailsDisabled
  style(host, {compound: [
      {disabled: true, paint: filter({ opacity: 50 }).cursor(not-allowed)},
    ]})
>
  <summary style(summary)><slot name="trigger"></slot></summary>
  <div>
    <div style(content)><slot></slot></div>
  </div>
</details>
