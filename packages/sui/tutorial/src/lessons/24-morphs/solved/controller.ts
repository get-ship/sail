import { controller, signal } from '@ship/sui-runtime'
import type { Scope } from './template.sui'

export default controller<Scope>({
  scope: () => {
    const name = signal('')
    return {
      name,
      add: async (label: string): Promise<void> => {
        const trimmed = label.trim()
        if (!trimmed) return
        const res = await fetch('/_deck/api/users/add', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ name: trimmed }),
        })
        if (res.ok) name.set('')
      },
    }
  },
})
