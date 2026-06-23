import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type Tone = 'info' | 'success' | 'warning' | 'destructive'

export default controller<Scope>({
  scope: () => ({ tone: signal<Tone>('info') }),
})
