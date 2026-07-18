<controller>
import { controller, signal, mount, type ControllerScope } from '@ship/sail-runtime'

type Scope = {}

export default controller((_ctx: ControllerScope<Scope>) => {
  const word = signal('a')
  mount((host) => {
    const t = setInterval(() => word.set(word() + 'x'), 10)
    return () => clearInterval(t)
  })
  return { word }
})
</controller>

<div data-testid="word">{word()}</div>
