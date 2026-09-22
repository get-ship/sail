<!--
  <accordion> — single disclosure. Native <details> owns open/close, keyboard,
  focus, and ARIA. Shared `name` = exclusive multi-item (native details name).
  Omit name for independent open; set `multiple` when several may stay open.

  Look: this tag owns its Style. No DOM.
  State: this file's <controller> (open/disabled as signals, onToggle callback).
  Listen: native `on:toggle` on <details>; disabled gate via `preventDefault`
  on click/keydown capture — not widget `use:` actions.
-->

<scope>
  props: {
    open = false
    disabled = false
    name = ''
    multiple = false
    onToggle: (detail: { open: boolean }) => void
  }

  // Shell + trigger from background / foreground / border. border style is
  // explicit: color+width alone leave UA border-style none → nothing paints.
  // overflow clip keeps open-content corners inside the host radius.
  style host = {display(block).border({ color: border, width: 1, style: solid }).overflow({ all: clip }).radius(lg).bg(background).type({ family: body })}
  // list(none) + after chevron: kill UA marker; single themed caret.
  style summary = {display(flex).pad({ y: 3, x: 4 }).fg(foreground).radius(lg).cursor(pointer).select(none).type({ size: sm, weight: semibold, list: none }), when: {hover: bg(hover-bg), after: margin({ s: auto }).size(4).content(empty).bg(muted-foreground)}, animate: { transition: play(colors, fast) }}
  style content = {pad({ x: 4, y: 3 })}
</scope>

<controller>
export default controller((init) => {
  function isOn(v) {
    const x = typeof v === 'function' ? v() : v
    return x === true || x === '' || x === 'true'
  }
  const disabled = derived(() => isOn(init.disabled))
  function onToggle(e) {
    const target = e && e.target
    const open = !!(target && typeof target.open === 'boolean' ? target.open : false)
    const cb = init.onToggle
    if (typeof cb === 'function') cb({ open })
  }
  function guardClick(e) {
    if (disabled()) e.preventDefault()
  }
  function guardKeydown(e) {
    if (!disabled()) return
    if (e.key === 'Enter' || e.key === ' ') e.preventDefault()
  }
  return { onToggle, guardClick, guardKeydown, disabled }
})
</controller>

<details
  on:toggle={onToggle}
  on:click={guardClick}
  on:keydown={guardKeydown}
  style(host, {compound: [
      {disabled: true, filter({ opacity: 50 }).cursor(not-allowed)},
    ]})
>
  <summary style(summary)><slot name="trigger"></slot></summary>
  <div>
    <div style(content)><slot></slot></div>
  </div>
</details>
