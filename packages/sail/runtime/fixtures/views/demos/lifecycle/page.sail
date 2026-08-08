<controller>
import { controller, signal, batch } from '@ship/sail-runtime'

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
      batch(() => {
        elapsed.set(n)
        log.set([stamp(n, `tick #${n}`), ...log()].slice(0, 8))
      })
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

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mono = { text: fontFamily(mono).fontSize(sm) }
</scope>

<column gap="4" style(page)>
  <p><a href="/demos">← demos</a></p>
  <column gap="1">
    <h1>lifecycle</h1>
    <p style(muted)>
      <code>mount:</code> runs after first DOM attach; return a cleanup
      callback to run on dispose. Navigate away to see <strong>unmount</strong>
      log to the console.
    </p>
  </column>

  <row gap="4" align="center">
    <span style(mono)>elapsed = <strong data-test="elapsed">{elapsed()}</strong>s</span>
    <button data-test="reset-log" variant="outline" on:click={clear}>clear log</button>
  </row>

  <ol data-test="log" style(mono)>
    <for {log() as line} key={line.k}>
      <li>{line.text}</li>
    </for>
  </ol>
</column>
