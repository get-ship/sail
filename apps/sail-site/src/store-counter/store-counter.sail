<!--
  Shared-state demo cell: every instance uses the same store (no count props).
-->
<scope>
  props: {
    label = 'Counter'
  }

  style labelText = { fg(muted).type(label) }
  style count = { type(heading) }
</scope>

<controller>
import { tally } from '@/stores/counter-store'

export default controller(() => {
  const { clicks, add } = useStore(tally)
  return { clicks, add }
})
</controller>

<card>
  <column gap="2" align="start">
    <small style(labelText)>{label()}</small>
    <row gap="3" align="center" nowrap>
      <button on:click={() => add()}>+1</button>
      <strong style(count)>{clicks()}</strong>
    </row>
  </column>
</card>
