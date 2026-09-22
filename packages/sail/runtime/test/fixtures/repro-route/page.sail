<controller>
import { controller, signal, type ControllerScope } from '@ship/sail-runtime'

type Scope = {}

export default controller((_ctx: ControllerScope<Scope>) => {
  const word = signal('a')
  let ticks = 0
  mount((host) => {
    host.setAttribute('data-mounted', 'true')
    const t = setInterval(() => {
      ticks += 1
      word.set(word() + 'x')
    }, 10)
    return () => {
      clearInterval(t)
      host.setAttribute('data-cleaned-up', 'true')
    }
  })
  return { word }
})
</controller>

<div data-testid="word">{word()}</div>
