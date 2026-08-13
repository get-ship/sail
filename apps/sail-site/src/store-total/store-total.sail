<!--
  Third reader of the shared tally store (State chapter demo).
-->
<scope>
  style line = { paint: ink({ color: muted-foreground }), text: type({ size: sm }) }
  style total = { paint: ink({ color: foreground }) }
</scope>

<controller>
import { tally } from '@/stores/counter-store'

export default controller(() => {
  const { clicks } = useStore(tally)
  return { clicks }
})
</controller>

<p style(line)>
  …and this line is a third component reading the same store:
  <strong style(total)>{clicks()} clicks</strong> total.
</p>
