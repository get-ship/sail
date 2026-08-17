<!--
  <demo-player> — nested-component demo: one `playing` field drives the
  equalizer (present only while playing) and a position clock effect that
  tracks playing so pause cleanup stops the timer.
-->
<scope>
  props: {
    playing = true
  }

  style card = { size(max) }
  style title = { type(body-sm) }
  style artist = { fg(muted).type(label) }
  style eq = { size(grow) }
  style bar = { size(1).radius(sm).bg(primary).origin(bottom), animate: { loop: play(eq) } }
  style barDim = { size(1).radius(sm).bg(primary).origin(bottom), animate: { loop: play(eq) } }
  style rest = { size(2).radius(full).bg(muted) }
  style meta = { size(grow) }
</scope>

<controller>
export default controller((ctx: { playing: () => boolean }) => {
  const clock = signal(23)
  effect(() => {
    if (!ctx.playing()) return
    interval(() => clock.set((p) => (p + 1) % 100), 150)
  })
  return { clock }
})
</controller>

<card style(card)>
  <row gap="3" align="center" nowrap>
    <column gap="1" style(meta)>
      <strong style(title)>Signals in the Rain</strong>
      <p style(artist)>The Reactive Quartet</p>
      <progress value={clock()} max="100"></progress>
    </column>
    <if {playing()}>
      <row gap="1" align="end" style(eq)>
        <div style(bar)></div>
        <div style(barDim)></div>
        <div style(bar)></div>
        <div style(barDim)></div>
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
