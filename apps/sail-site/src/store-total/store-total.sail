<!--
  Third reader of the shared tally store (State chapter demo).
-->
<scope>
  style line = { paint: fg(muted-foreground), text: fontSize(sm) }
  style total = { paint: fg(foreground) }
</scope>

<controller>
import { tally } from '../stores/counter-store'

export default controller(() => {
  const { clicks } = useStore(tally)
  return { clicks }
})
</controller>

<p style(line)>
  …and this line is a third component reading the same store:
  <strong style(total)>{clicks()} clicks</strong> total.
</p>
