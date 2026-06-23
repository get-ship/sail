import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface LogLine { k: number; text: string }

const stamp = (k: number, text: string): LogLine => ({
  k,
  text: `${new Date().toLocaleTimeString()} — ${text}`,
})

export default controller<Scope>({
  scope: () => {
    const elapsed = signal(0)
    const log = signal<LogLine[]>([])
    return {
      elapsed,
      log,
      clear: () => log.set([]),
    }
  },

  mount: (state) => {
    state.log = [stamp(0, 'mount'), ...state.log]
    let n = 0
    const timer = setInterval(() => {
      n += 1
      state.elapsed = n
      state.log = [stamp(n, `tick #${n}`), ...state.log].slice(0, 8)
    }, 1000)
    return () => {
      clearInterval(timer)
      console.info('[demo:lifecycle] unmount')
    }
  },
})
