<!--
  <demo-player> — the intro's nested-component demo. A compact music-player
  card driven entirely by one field, `playing`. The equalizer bars only exist
  (and loop) while playing() holds them in the DOM; the position clock is an
  effect that TRACKS playing(), so pausing re-runs the effect and its cleanup
  stops the timer. Lifted from the showcase player, shrunk to embed.
-->
<scope>
  props: {
    playing = true
  }

  const card = style({ box: pad(3).w(full) })
    const title = style({ text: fontSize(sm).truncate() })
  const artist = style({ paint: fg(muted-foreground), text: fontSize(xs).truncate() })
  const eq = style({ box: minH(6) })
  const bar = style({ box: w(1), paint: radius(sm).bg(primary).origin(bottom) })
  const rest = style({ box: w(1).h(2), paint: radius(full).bg(muted-foreground).opacity(50) })
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

<card>
  style(card)

  <row gap="3" align="center" nowrap>
    <column gap="1" grow min-w="0">
      <strong>
        style(title)
Signals in the Rain</strong>
      <p>
        style(artist)
The Reactive Quartet</p>
      <progress value={position()} max="100"></progress>
    </column>
    <if {playing()}>
      <row gap="1" align="end">
        style(eq)

        <div>
          style([bar, style({ box: h(4), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })])
</div>
        <div>
          style([bar, style({ box: h(6), paint: opacity(75), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })])
</div>
        <div>
          style([bar, style({ box: h(3), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })])
</div>
        <div>
          style([bar, style({ box: h(5), paint: opacity(75), animate: loop({ scale-y: 'spring:gentle', stagger: '0.1s' }) })])
</div>
      </row>
    <else>
      <row gap="1" align="center">
        style(eq)

        <div>
          style(rest)
</div>
        <div>
          style(rest)
</div>
        <div>
          style(rest)
</div>
        <div>
          style(rest)
</div>
      </row>
    </else></if>
  </row>
</card>
