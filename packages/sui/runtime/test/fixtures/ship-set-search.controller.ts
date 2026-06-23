import { controller, signal, derived } from '@ship/sui-runtime'

const ITEMS = ['apple', 'apricot', 'banana', 'blueberry', 'cherry', 'grape']

export default controller({
  scope: () => {
    const query = signal('')
    const caseSensitive = signal(false)
    const filtered = derived(() => {
      const q = query()
      const cs = caseSensitive()
      if (!q) return ITEMS
      return ITEMS.filter((item) =>
        cs ? item.includes(q) : item.toLowerCase().includes(q.toLowerCase()),
      )
    })
    return { query, caseSensitive, filtered }
  },
})
