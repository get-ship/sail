<!-- query()/mutation() derived-identity proof against the crm-demo composition. -->

<controller>
import { controller, query, mutation, signal } from '@ship/sail-runtime'
import { crm } from '../../src/compositions/crm-demo'

export default controller(() => {
  const stage = signal<'open' | 'closed'>('open')

  // Thunk receives the cache AbortSignal and forwards it to the SDK fetch.
  // Compiler key derivation reads the call expression (member path + stage()).
  const deals = query((abortSignal) => crm.deals.list({ stage: stage() }, abortSignal))

  const close = mutation((abortSignal) => crm.deals.update({ id: 'd1', stage: 'closed' }, abortSignal))

  return { stage, deals, close }
})
</controller>

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style lead = { paint: fg(muted-foreground), text: fontSize(sm) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style error = { paint: fg(destructive), text: fontSize(sm) }
  style list = { box: pad(0).m(0), paint: list(none) }
</scope>

<column gap="3" style(page)>
  <h1 style(title)>query() demo</h1>
  <p style(lead)>A real query()/mutation() round-trip against crm-demo.</p>

  <row gap="2">
    <button on:click={() => stage.set('open')} aria-pressed={stage() === 'open'}>open</button>
    <button on:click={() => stage.set('closed')} aria-pressed={stage() === 'closed'}>closed</button>
  </row>

  <match {deals().kind}>
    {loading =>
      <p style(muted)>Loading…</p>
    }
    {error =>
      <p style(error)>Error</p>
    }
    {ok =>
      <column gap="1" style(list)>
        <for {deals().value.items as deal} key={deal.id}>
          <li>{deal.id} — {deal.stage} — {deal.amount}</li>
        </for>
      </column>
    }
  </match>

  <button on:click={() => close.mutate()}>close d1</button>
  <p>{close.state().kind}</p>
</column>
