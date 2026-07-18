<!--
  <store-counter> — one cell of the State chapter's shared-state demo. Every
  instance resolves the SAME store; no props carry the count, no events
  bubble it. Click any instance, all of them move.
-->
<scope>
  props: {
    label = 'Counter'
  }
</scope>

<controller>
import { controller, useStore } from '@ship/sail-runtime'
import { tally } from './counter-store'

export default controller(() => {
  const { clicks, add } = useStore(tally)
  return { clicks, add }
})
</controller>

<card sx="p-4">
  <column gap="2" align="start">
    <small sx="text-xs uppercase tracking-wide text-muted-foreground">{label()}</small>
    <row gap="3" align="center" nowrap>
      <button on:click={() => add()}>+1</button>
      <strong sx="text-lg">{clicks()}</strong>
    </row>
  </column>
</card>
