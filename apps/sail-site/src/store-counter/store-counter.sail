<!--
  <store-counter> — one cell of the State chapter's shared-state demo. Every
  instance resolves the SAME store; no props carry the count, no events
  bubble it. Click any instance, all of them move.
-->
<scope>
  props: {
    label = 'Counter'
  }

  style card = { box: pad(4) }
  style labelText = { paint: fg(muted-foreground), text: fontSize(xs).textCase(uppercase).tracking(wide) }
  style count = { text: fontSize(lg) }
</scope>

<controller>
import { useStore } from '@ship/sail-runtime'
import { tally } from '../compositions/counter-store'

export default controller(() => {
  const { clicks, add } = useStore(tally)
  return { clicks, add }
})
</controller>

<card style(card)>
  <column gap="2" align="start">
    <small style(labelText)>{label()}</small>
    <row gap="3" align="center" nowrap>
      <button on:click={() => add()}>+1</button>
      <strong style(count)>{clicks()}</strong>
    </row>
  </column>
</card>
