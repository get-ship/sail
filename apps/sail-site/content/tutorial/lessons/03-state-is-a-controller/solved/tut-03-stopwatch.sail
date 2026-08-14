<!--
  Lesson 03 solved — a stopwatch.

  The <controller> block below owns the state; the template binds only
  what it returns: clock, running, laps, and three handlers.
-->

<scope>
  style face = { pad(6).bg(surface-1).border({ color: border, width: 1 }).radius(lg) }
  style clockFace = { type({ size: 4xl, tracking: tight }) }
  style lapRow = { pad({ x: 3, y: 1 }).bg(surface-1).radius(md) }
  style lapMeta = { fg(muted-foreground) }
  style lapAt = { type({ size: sm }) }
</scope>

<controller>
interface Lap {
  id: number
  at: string
}

// The controller owns ALL state. The template can bind only what this
// function returns — nothing else exists on the other side of the boundary.
export default controller(() => {
  const elapsed = signal(0) // milliseconds
  const running = signal(false)
  const laps = signal<Lap[]>([])

  const clock = derived(() => {
    const total = elapsed()
    const m = Math.floor(total / 60000)
    const s = Math.floor((total % 60000) / 1000)
    const tenths = Math.floor((total % 1000) / 100)
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}.${tenths}`
  })

  // Interval is owned by this effect: re-runs when `running` flips; the
  // returned cleanup runs before every re-run and on unmount.
  effect(() => {
    if (!running()) return
    const id = setInterval(() => elapsed.set((ms) => ms + 100), 100)
    return () => clearInterval(id)
  })

  let lapId = 0

  return {
    clock,
    running,
    laps,
    toggle: (): void => running.set(!running()),
    lap: (): void => laps.set([{ id: ++lapId, at: clock() }, ...laps()]),
    reset: (): void => {
      batch(() => {
        running.set(false)
        elapsed.set(0)
        laps.set([])
      })
    },
  }
})
</controller>

<column gap="4">
  <center style(face)>
    <strong style(clockFace)>{clock()}</strong>
  </center>

  <row gap="2" justify="center">
    <button variant="solid" tone={running() ? 'warning' : 'primary'} on:click={() => toggle()}>
      {running() ? 'Pause' : 'Start'}
    </button>
    <button variant="outline" tone="primary" disabled={!running()} on:click={() => lap()}>Lap</button>
    <button variant="ghost" on:click={() => reset()}>Reset</button>
  </row>

  <if {laps().length > 0}>
    <column gap="1">
      <for {laps() as lap} key={lap.id}>
        <row align="center" justify="between" style(lapRow)>
          <small style(lapMeta)>lap {lap.id}</small>
          <strong style(lapAt)>{lap.at}</strong>
        </row>
      </for>
    </column>
  </if>
</column>
