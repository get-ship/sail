import { controller, signal, effect, batch, untrack } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const cpu = signal(0)
    const mem = signal(0)
    const req = signal(0)
    const stamp = signal(0)

    // Tick once per second; batch the three writes so subscribers
    // see one update instead of three.
    effect(() => {
      const id = window.setInterval(() => {
        batch(() => {
          cpu.set(Math.round(Math.random() * 100))
          mem.set(Math.round(Math.random() * 1024))
          req.set(Math.round(Math.random() * 500))
        })
        // stamp is read-only context for the logger — set without batching.
        stamp.set(Date.now())
      }, 1000)
      return () => window.clearInterval(id)
    })

    // Logs once per batched tick, even though three signals changed.
    // untrack(stamp) means we read it without subscribing — stamp
    // changes alone don't re-fire this effect.
    effect(() => {
      const s = untrack(() => stamp())
      console.log(`tick @${s}: cpu=${cpu()} mem=${mem()} req=${req()}`)
    })

    return { cpu, mem, req, stamp }
  },
})
