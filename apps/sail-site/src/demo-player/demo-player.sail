<!--
  <demo-player> — nested-component demo: one `playing` field drives the
  equalizer (present only while playing) and a position clock effect that
  tracks playing so pause cleanup stops the timer.
-->
<scope>
  props: {
    playing = true
  }

  style card = { box: space({ pad: 3 }).size({ w: full }) }
  style title = { text: type({ size: sm, wrap: truncate }) }
  style artist = { paint: ink({ color: muted-foreground }), text: type({ size: xs, wrap: truncate }) }
  style eq = { box: size({ min-h: 6 }) }
  style bar = { box: size({ w: 1 }), paint: radius(sm).fill({ color: primary }).motion({ origin: bottom }) }
  style barDim = { box: size({ w: 1 }), paint: radius(sm).fill({ color: primary }).filter({ opacity: 75 }).motion({ origin: bottom }) }
  style rest = { box: size({ w: 1, h: 2 }), paint: radius(full).fill({ color: muted-foreground }).filter({ opacity: 50 }) }
  style meta = { box: flow({ grow: 1 }).size({ min-w: 0 }) }
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
        <div style(bar, { box: size({ h: 4 }), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })></div>
        <div style(barDim, { box: size({ h: 6 }), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })></div>
        <div style(bar, { box: size({ h: 3 }), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })></div>
        <div style(barDim, { box: size({ h: 5 }), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })></div>
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
