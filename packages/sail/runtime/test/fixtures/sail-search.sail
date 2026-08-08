<controller>
import { controller, signal, derived } from '@ship/sail-runtime'

const ITEMS = ['apple', 'apricot', 'banana', 'blueberry', 'cherry', 'grape']

export default controller(() => {
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
})
</controller>

<div>
  <input data-testid="query" bind:value={query} />
  <input data-testid="case" type="checkbox" bind:checked={caseSensitive} />
  <if {filtered().length === 0}>
    <p data-testid="empty">No results for {query()}</p>
  </if>
  <for {filtered() as item} key={item}>
    <span data-testid="result">{item}</span>
  </for>
</div>
