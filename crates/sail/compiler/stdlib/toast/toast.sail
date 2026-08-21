<!--
  <toast> — auto-dismiss + close. Style value language only.
  tone defaults to info — a bare <toast> is a finished notice.
  Semantic paint = bound tone group paths.

  State: this file's <controller> (duration timer with hover/focus pause,
  open signal, close). Listen: native <button> for the X; on:pointer* /
  on:focus* pause+resume the timer. duration ≤ 0 is sticky (no timer).
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'danger' | 'accent' | 'muted' = 'info'
    duration = ''
    onDismiss: () => void
  }

  // Rest wash is the bound tone at mix 15 (same spend as chip soft / button soft).
  // Stripe is the solid tone rail. Do not drop either when a compound fails.
  style base = {position(relative).overflow({ all: hidden }).pad({ y: 3, e: 4, s: 5 }).fg(foreground).bg({ color: tone, mix: 15 }).radius(md).elevation(md).border({ color: border, width: 1 }).pointerEvents(auto).type({ family: body, size: sm })}
  style stripe = {position({ scheme: absolute, l: 0, t: 0, b: 0 }).size({ w: 1 }).content(empty).bg(primary)}
  // One cell for the notice — a wrapping row would stack <strong>, the
  // sentence, and the close as sibling flex lines (the "toast is a mess" look).
  style body = {grow(1).size({ min-w: 0 })}
  style close = {display(inline-flex).size({ w: control-h-xs, h: control-h-xs }).pad(0).fg(foreground).bg(transparent).border({ width: 0 }).radius(sm).filter({ opacity: 70 }).cursor(pointer).type({ leading: none, align: center })}
  style closeGlyph = {size({ w: 100%, h: 100% })}
</scope>

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  const raw = read(init.duration)
  const n = Number(raw)
  const duration = raw == null || raw === '' ? 5000 : Number.isFinite(n) ? n : 5000
  const open = signal(true)
  let timer
  function clear() {
    if (timer) {
      clearTimeout(timer)
      timer = undefined
    }
  }
  function fire() {
    clear()
    open.set(false)
    const cb = read(init.onDismiss)
    if (typeof cb === 'function') cb()
  }
  function start() {
    if (!open() || duration <= 0) return
    clear()
    timer = setTimeout(fire, duration)
  }
  function pause() {
    clear()
  }
  start()
  onCleanup(clear)
  return { open, close, start, pause }
})
</controller>

<if open>
  <row
    nowrap
    align="center"
    gap="3"
    on:pointerenter={pause}
    on:pointerleave={start}
    on:focusin={pause}
    on:focusout={start}
    style(base, {when: {before: style(stripe, {bg(tone)})}, compound: [
      {tone: primary, bg({ color: primary, mix: 15 }).border({ color: primary })},
      {tone: info, bg({ color: info, mix: 15 }).border({ color: info })},
      {tone: success, bg({ color: success, mix: 15 }).border({ color: success })},
      {tone: warning, bg({ color: warning, mix: 15 }).border({ color: warning })},
      {tone: danger, bg({ color: danger, mix: 15 }).border({ color: danger })},
      {tone: accent, bg({ color: accent, mix: 15 }).border({ color: accent })},
      {tone: muted, bg({ color: muted, mix: 15 }).border({ color: border })},
    ]})
  >
    <div style(body)>
      <slot></slot>
    </div>
    <button
      type="button"
      anatomy
      label="Dismiss"
      on:click={close}
      style(close, {shrink(0).grow(0), when: {hover: fg(foreground).bg(surface).filter({ opacity: 100 })}})
    >
      <row inline nowrap align="center" justify="center" style(closeGlyph)>✕</row>
    </button>
  </row>
</if>
