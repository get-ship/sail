<!--
  <demo-player> — nested-component demo: one `playing` field drives the
  equalizer (present only while playing) and a position clock effect that
  tracks playing so pause cleanup stops the timer.
-->
<scope>
  props: {
    playing = true
  }

  style card = { box: pad(3).w(full) }
  style title = { text: fontSize(sm).truncate() }
  style artist = { paint: fg(muted-foreground), text: fontSize(xs).truncate() }
  style eq = { box: minH(6) }
  style bar = { box: w(1), paint: radius(sm).bg(primary).origin(bottom) }
  style barDim = { box: w(1), paint: radius(sm).bg(primary).opacity(75).origin(bottom) }
  style rest = { box: w(1).h(2), paint: radius(full).bg(muted-foreground).opacity(50) }
  style meta = { box: grow(1).minW(0) }
</scope>

<controller>
import type { Scope } from './demo-player.sail'
import { interval } from '@ship/sail-runtime'

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
        <div style(bar, { box: h(4), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })></div>
        <div style(barDim, { box: h(6), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })></div>
        <div style(bar, { box: h(3), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })></div>
        <div style(barDim, { box: h(5), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })></div>
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
