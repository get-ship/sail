<!--
  Lesson 03 starter — a stopwatch whose clock never moves.

  The <controller> block below owns the state.
  The template binds only what that controller function returns.
-->

<controller>
// The controller owns ALL state. The template can bind only what this
// function returns — nothing else exists on the other side of the boundary.
export default controller(() => {
  const elapsed = signal(0) // milliseconds
  const running = signal(false)

  const clock = derived(() => {
    const total = elapsed()
    const m = Math.floor(total / 60000)
    const s = Math.floor((total % 60000) / 1000)
    const tenths = Math.floor((total % 1000) / 100)
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}.${tenths}`
  })

  // TODO 1: the clock never ticks. Add an effect that starts a
  // setInterval bumping `elapsed` by 100 while running() is true —
  // and RETURN the clearInterval cleanup. The effect re-runs when
  // `running` flips; the cleanup runs before each re-run and when
  // the component unmounts, so the timer can never leak.

  // TODO 2: add a `laps` signal (array of { id, at }) and a `lap()`
  // handler that prepends the current clock() reading.

  return {
    clock,
    running,
    toggle: (): void => running.set(!running()),
    reset: (): void => {
      batch(() => {
        running.set(false)
        elapsed.set(0)
      })
    },
  }
})
</controller>

<scope>
  style face = { pad(6).bg(surface-1).border({ color: border, width: 1 }).radius(lg) }
  style clockFace = { type({ size: 4xl, tracking: tight }) }
</scope>

<column gap="4">
  <center style(face)>
    <strong style(clockFace)>{clock()}</strong>
  </center>

  <row gap="2" justify="center">
    <button variant="solid" tone={running() ? 'warning' : 'primary'} on:click={() => toggle()}>
      {running() ? 'Pause' : 'Start'}
    </button>
    <button variant="ghost" on:click={() => reset()}>Reset</button>
    <!-- TODO 3: add a Lap button (disabled unless running()) and a keyed
         {#for} list of laps below. -->
  </row>
</column>
