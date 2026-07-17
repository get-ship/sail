<controller>
import { controller, signal, mount, type ViewScope } from '@ship/sail-runtime'

type Scope = {}

export default controller((_ctx: ViewScope<Scope>) => {
  const word = signal('a')
  mount((host) => {
    const t = setInterval(() => word.set(word() + 'x'), 10)
    return () => clearInterval(t)
  })
  return { word }
})
</controller>

<div data-testid="word">{word()}</div>
