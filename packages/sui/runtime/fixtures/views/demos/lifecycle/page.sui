<controller>
import { controller, signal } from '@ship/sail-runtime'

interface LogLine { k: number; text: string }

const stamp = (k: number, text: string): LogLine => ({
  k,
  text: `${new Date().toLocaleTimeString()} — ${text}`,
})

export default controller(() => {
  const elapsed = signal(0)
  const log = signal<LogLine[]>([])

  mount(() => {
    log.set([stamp(0, 'mount'), ...log()])
    let n = 0
    const timer = setInterval(() => {
      n += 1
      elapsed.set(n)
      log.set([stamp(n, `tick #${n}`), ...log()].slice(0, 8))
    }, 1000)
    return () => {
      clearInterval(timer)
      console.info('[demo:lifecycle] unmount')
    }
  })

  return {
    elapsed,
    log,
    clear: () => log.set([]),
  }
})
</controller>

<div data-demo="true">
  <p><a href="/demos">← demos</a></p>
  <h1>lifecycle</h1>
  <p>
    <code>mount:</code> runs after first DOM attach; return a cleanup
    callback to run on dispose. Navigate away to see <strong>unmount</strong>
    log to the console.
  </p>

  <section>
    <span>elapsed = <strong data-test="elapsed">{elapsed()}</strong>s</span>
    <button data-test="reset-log" on:click={clear}>clear log</button>
  </section>

  <ol data-test="log">
    {#for line of log() key={line.k}}
      <li>{line.text}</li>
    {/for}
  </ol>
</div>
