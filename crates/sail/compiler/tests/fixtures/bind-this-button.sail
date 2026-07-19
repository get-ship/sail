<controller>
import { controller, signal } from '@ship/sail-runtime'
export default controller((ctx) => {
  const btnEl = signal<HTMLButtonElement | null>(null)
  return { btnEl }
})
</controller>
<button bind:this={btnEl}>click</button>
