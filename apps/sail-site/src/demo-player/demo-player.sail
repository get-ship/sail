<!--
  <demo-player> — nested-component demo: one `playing` field drives the
  equalizer (present only while playing) and a position clock effect that
  tracks playing so pause cleanup stops the timer.
-->
<scope>
  props: {
    playing = true
  }

  style card = { pad(3).size({ w: full }) }
  style title = { type({ size: sm, wrap: truncate }) }
  style artist = { fg(muted-foreground).type({ size: xs, wrap: truncate }) }
  style eq = { size({ min-h: 6 }) }
  style bar = { size({ w: 1 }).radius(sm).bg(primary).origin(bottom) }
  style barDim = { size({ w: 1 }).radius(sm).bg(primary).filter({ opacity: 75 }).origin(bottom) }
  style rest = { size({ w: 1, h: 2 }).radius(full).bg(muted-foreground).filter({ opacity: 50 }) }
  style meta = { grow(1).size({ min-w: 0 }) }
</scope>

<controller>
import type { Scope } from './demo-player.sail'

export default controller((ctx: Scope) => {
  const position = signal(23)
  effect(() => {
    if (!ctx.playing()) return
    interval(() => position.set((p) => (p + 1) % 100), 150)
  })
  return { position }
})
</controller>

<card style(card)>
  <row gap="3" align="center" nowrap>
    <column gap="1" style(meta)>
      <strong style(title)>Signals in the Rain</strong>
      <p style(artist)>The Reactive Quartet</p>
      <progress value={position()} max="100"></progress>
    </column>
    <if {playing()}>
      <row gap="1" align="end" style(eq)>
        <div style(bar, { size({ h: 4 }), animate: { loop: play(fade) } })></div>
        <div style(barDim, { size({ h: 6 }), animate: { loop: play(fade) } })></div>
        <div style(bar, { size({ h: 3 }), animate: { loop: play(fade) } })></div>
        <div style(barDim, { size({ h: 5 }), animate: { loop: play(fade) } })></div>
      </row>
    {else =>
      <row gap="1" align="center" style(eq)>
        <div style(rest)></div>
        <div style(rest)></div>
        <div style(rest)></div>
        <div style(rest)></div>
      </row>
    }</if>
  </row>
</card>
