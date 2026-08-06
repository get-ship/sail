<scope>
  props: {
    title = ''
    body = ''
    tags = '[]'
  }
</scope>
<controller>
import { controller, signal, derived } from '@ship/sail-runtime'

export default controller(({ title, body, tags }: { title: () => string; body: () => string; tags: () => string }) => {
    const expanded = signal(false)
    const parsedTags = derived(() => {
      try {
        const val = JSON.parse(tags())
        return Array.isArray(val) ? (val as string[]) : []
      } catch {
        return []
      }
    })
    return { expanded, parsedTags }
  })
</controller>

<div>
  <span data-testid="title">{title()}</span>
  <button data-testid="toggle" on:click={() => expanded.set(!expanded())}>toggle</button>
  <if {expanded()}>
    <div data-testid="body">{body()}</div>
    <ul data-testid="tags">
      <for {parsedTags() as tag} key={tag}>
        <li>{tag}</li>
      </for>
    </ul>
  </if>
</div>
