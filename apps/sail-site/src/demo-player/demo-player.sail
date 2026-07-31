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

  const card = style.pad(3).w(full)
    const title = style.fontSize(sm).truncate()
  const artist = style.fontSize(xs).fg(muted-foreground).truncate()
  const eq = style.minH(6)
  const bar = style.w(1).radius(sm).bg(primary).origin(bottom)
  const rest = style.w(1).h(2).radius(full).bg(muted-foreground).opacity(50)
</scope>

<controller>
import type { Scope } from './demo-player.sail'

export default controller((ctx: Scope) => {
    const position = signal(23)
    effect(() => {
      if (!ctx.playing()) return
      const t = setInterval(() => position.set((p) => (p + 1) % 100), 150)
      return () => clearInterval(t)
    })
    return { position }
})
</controller>

<card sx={card}>
  <row gap="3" align="center" nowrap>
    <column gap="1" grow min-w="0">
      <strong sx={title}>Signals in the Rain</strong>
      <p sx={artist}>The Reactive Quartet</p>
      <progress value={position()} max="100"></progress>
    </column>
    <if {playing()}>
      <row gap="1" align="end" sx={eq}>
        <div sx={[bar, style.h(4).loop({ scale-y: 'spring:gentle', stagger: '0.1s' })]}></div>
        <div sx={[bar, style.h(6).opacity(75).loop({ scale-y: 'spring:gentle', stagger: '0.1s' })]}></div>
        <div sx={[bar, style.h(3).loop({ scale-y: 'spring:gentle', stagger: '0.1s' })]}></div>
        <div sx={[bar, style.h(5).opacity(75).loop({ scale-y: 'spring:gentle', stagger: '0.1s' })]}></div>
      </row>
    <else>
      <row gap="1" align="center" sx={eq}>
        <div sx={rest}></div>
        <div sx={rest}></div>
        <div sx={rest}></div>
        <div sx={rest}></div>
      </row>
    </else></if>
  </row>
</card>
