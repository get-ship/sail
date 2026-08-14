<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => {
  const n = signal(0)
  const save = () => n.set(n() + 1)
  return { n, save }
})
</controller>
<button label="Save" on:click={save}>Save</button>
<p>count {n()}</p>
