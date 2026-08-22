<!--
  <alert> — L1 inline status banner. Style value language only.
  tone defaults to info — a bare <alert> is a finished status banner.
  Semantic paint = bound tone group paths.
  Host subset: muted | info | success | warning | danger (no primary/accent product use).

  State: this file's <controller> (open + dismissible signals, dismiss).
  Listen: native <button> X, gated on dismissible via <if>. Dismiss fires
  onDismiss; the banner goes hidden + display(none) (element mode needs an
  element root). role=status comes from HOST_ANATOMY.
-->

<scope>
  props: {
    tone: 'muted' | 'info' | 'success' | 'warning' | 'danger' = 'info'
    variant: 'default' | 'banner' = 'default'
    dismissible = false
    onDismiss: () => void
  }

  style base = {position(relative).overflow({ all: hidden }).pad(3).group().border({ color: border, width: 1 }).radius(md).bg(muted).fg(foreground).elevation(xs).type({ family: body, size: base, leading: normal })}
  // ::before stripe geometry (rest fill primary; host attaches tone via when.before).
  style stripe = {position({ scheme: absolute, l: 0, t: 0, b: 0 }).size({ w: 1 }).content(empty).bg(primary)}
  style variantDefault = {radius(md).elevation(xs)}
  // Full-bleed: flush radius, no lift, stretch width, thicker start stripe.
  style variantBanner = {radius(none).elevation(none).size({ w: 100% }).self(stretch)}
  style ico = {size(control-h-xs).radius(md).bg(muted).fg(primary)}
  style title = {type({ weight: semibold })}
  style body = {fg(foreground)}
  style actions = {margin({ t: 2 })}
  style close = {display(none).self(start).size(control-h-xs).pad(0).border({ width: 0 }).radius(sm).bg(transparent).fg(primary).cursor(pointer), animate: { transition: play(colors, fast) }}
  style xmark = {display(block).size(icon-size-sm)}
  style growCol = {grow(1).size({ min-w: 0 })}
  style gone = {display(none).pointerEvents(none)}
</scope>

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  function isOn(v) {
    const x = read(v)
    return x === true || x === '' || x === 'true'
  }
  const dismissible = derived(() => isOn(init.dismissible))
  const open = signal(true)
  function dismiss() {
    if (!dismissible() || !open()) return
    open.set(false)
    const cb = read(init.onDismiss)
    if (typeof cb === 'function') cb()
  }
  const closed = derived(() => !open())
  return { dismissible, open, closed, dismiss }
})
</controller>

<div hidden={closed()} style({match: { open: { false: gone } }})>
  <row
    nowrap
    align="start"
    gap="3"
    style(base, {when: {before: style(stripe, {bg(tone)}), forced-colors: {border({ color: CanvasText }), when: {before: bg(CanvasText)}}}, match: { variant: { default: variantDefault, banner: variantBanner } }, compound: [
      {tone: muted, when: {before: bg(primary)}},
      {variant: banner, when: {before: size({ w: 2 })}},
    ]})
>
  <if {open()}>
    <div style({when: {empty: display(none)}})>
    <row
      inline
      nowrap
      align="center"
      justify="center"
      style(ico, {shrink(1).fg(tone), compound: [{tone: muted, bg(muted).fg(primary)}]})
    >
      <slot name="icon"></slot>
    </row>
    </div>
    <column gap="1" style(growCol)>
      <div style(title, {when: {empty: display(none)}})>
        <slot name="title"></slot>
      </div>
      <div style(body)>
        <slot></slot>
      </div>
      <div style({when: {empty: display(none)}})>
      <row align="center" gap="2" style(actions)>
        <slot name="actions"></slot>
      </row>
      </div>
    </column>
    <if {dismissible()}>
      <button
        type="button"
        anatomy
        label="Dismiss"
        on:click={dismiss}
        style(close, {shrink(1), when: {group-dismissible: display(inline-block), hover: bg(muted).fg(foreground), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}})
      >
        <svg viewBox="0 0 10 10" hidden="true" stroke-width="1.75" stroke-linecap="round" stroke="currentColor" fill="none" style(xmark)>
          <path d="M2 2 L8 8 M8 2 L2 8"></path>
        </svg>
      </button>
      </if>
    </if>
  </row>
</div>
