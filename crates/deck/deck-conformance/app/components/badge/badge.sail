<!-- Component-with-controller proof: signal count + bump, same setup(ctx) path as routes. -->

<controller>
import { controller, signal } from '@ship/sail-runtime'

export default controller(() => {
  const count = signal(0)
  const bump = (): void => count.set(count() + 1)
  return { count, bump }
})
</controller>

<scope>
  style host = { pad(1).radius(full).bg(surface-2).fg(foreground) }
  style mono = { display(contents).type({ size: sm, family: mono }) }
</scope>

<row gap="1" align="center" justify="center" style(host)>
  <span style(mono)>
    <button on:click={() => bump()}>{count()}</button>
  </span>
</row>
