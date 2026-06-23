import { controller, signal, derived } from '@ship/sui-runtime'

export default controller({
  scope: ({ title, body, tags }: { title: () => string; body: () => string; tags: () => string }) => {
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
  },
})
