<!--
  <store-total> — the third reader in the State chapter's shared-state demo.
  A DIFFERENT component from <store-counter>, resolving the same store: the
  point is that sharing needs no wiring between them.
-->
<controller>
import { useStore } from '@ship/sail-runtime'
import { tally } from '../store-counter/counter-store'

export default controller(() => {
  const { clicks } = useStore(tally)
  return { clicks }
})
</controller>

<p sx={style.fontSize(sm).fg(muted-foreground)}>
  …and this line is a third component reading the same store:
  <strong sx={style.fg(foreground)}>{clicks()} clicks</strong> total.
</p>
