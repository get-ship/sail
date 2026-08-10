// Browser bundle entry for the query/mutation e2e (spike S6.1).
//
// Mirrors crates/deck/deck-conformance/app/routes/query-demo/page.sail
// behavior as a product UI into #root, driven by real queryKeyed + mutation
// (uncompiled public APIs). In-memory delayed data port — never mockQueryState
// as the assertion substrate.
//
// CONTRACT: see e2e/query-mutation.spec.ts header.

import { signal, effect } from '../src/signal'
import { createScope, withScope } from '../src/effect-scope'
import { queryKeyed, mutation } from '../src/query'

type Stage = 'open' | 'closed'

interface Deal {
  id: string
  stage: Stage
  amount: number
}

/** Seed matches the gate: open d1/d2, closed-only d3; close d1 mutates stage. */
const dealsDb: Deal[] = [
  { id: 'd1', stage: 'open', amount: 4200 },
  { id: 'd2', stage: 'open', amount: 9100 },
  { id: 'd3', stage: 'closed', amount: 1500 },
]

let firstOpenFetch = true

function sleep(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(new DOMException('Aborted', 'AbortError'))
      return
    }
    const t = setTimeout(resolve, ms)
    signal?.addEventListener(
      'abort',
      () => {
        clearTimeout(t)
        reject(new DOMException('Aborted', 'AbortError'))
      },
      { once: true },
    )
  })
}

/** In-memory list port — ≥100ms on first open fetch so Loading… paints. */
async function listDeals(
  stage: Stage,
  abortSignal?: AbortSignal,
): Promise<{ items: Deal[] }> {
  if (stage === 'open' && firstOpenFetch) {
    firstOpenFetch = false
    await sleep(150, abortSignal)
  } else {
    // Keep stage re-keys async so a loading flash is possible under the poll.
    await sleep(40, abortSignal)
  }
  return {
    items: dealsDb.filter((d) => d.stage === stage).map((d) => ({ ...d })),
  }
}

async function closeDealD1(abortSignal?: AbortSignal): Promise<Deal> {
  await sleep(30, abortSignal)
  const deal = dealsDb.find((d) => d.id === 'd1')
  if (!deal) throw new Error('deal d1 not found')
  deal.stage = 'closed'
  return { ...deal }
}

function mountQueryDemo(): void {
  const host = document.getElementById('root')
  if (!host) throw new Error('[query-mutation-e2e] #root missing')

  const scope = createScope()
  withScope(scope, () => {
    const stage = signal<Stage>('open')

    // Explicit-key read: stage() is tracked so filter changes re-key / refetch.
    const deals = queryKeyed(
      () => ['crm', 'deals', 'list', stage()] as const,
      (abortSignal) => listDeals(stage(), abortSignal),
      { retry: false },
    )

    // Auto-invalidate the crm.deals.* namespace on success (compiler-shaped).
    const close = mutation(
      (abortSignal) => closeDealD1(abortSignal),
      undefined,
      ['crm', 'deals'],
    )

    const root = document.createElement('div')
    root.setAttribute('data-testid', 'query-demo')

    const openBtn = document.createElement('button')
    openBtn.type = 'button'
    openBtn.textContent = 'open'
    openBtn.addEventListener('click', () => stage.set('open'))

    const closedBtn = document.createElement('button')
    closedBtn.type = 'button'
    closedBtn.textContent = 'closed'
    closedBtn.addEventListener('click', () => stage.set('closed'))

    const filterRow = document.createElement('div')
    filterRow.append(openBtn, closedBtn)

    const queryStatus = document.createElement('div')
    queryStatus.setAttribute('data-testid', 'query-status')

    const body = document.createElement('div')

    const closeBtn = document.createElement('button')
    closeBtn.type = 'button'
    closeBtn.textContent = 'close d1'
    closeBtn.addEventListener('click', () => {
      void close.mutate().catch(() => {
        /* state signal carries error; gate does not assert rejection */
      })
    })

    const mutationStatus = document.createElement('div')
    mutationStatus.setAttribute('data-testid', 'mutation-status')
    mutationStatus.setAttribute('data-kind', 'idle')

    root.append(filterRow, queryStatus, body, closeBtn, mutationStatus)
    host.replaceChildren(root)

    // Mirror page.sail: stage buttons, query match arms, mutation kind.
    effect(() => {
      const s = stage()
      openBtn.setAttribute('aria-pressed', s === 'open' ? 'true' : 'false')
      closedBtn.setAttribute('aria-pressed', s === 'closed' ? 'true' : 'false')
    })

    effect(() => {
      const state = deals()
      queryStatus.setAttribute('data-kind', state.kind)
      body.replaceChildren()

      if (state.kind === 'loading') {
        const p = document.createElement('p')
        p.textContent = 'Loading…'
        body.append(p)
        return
      }

      if (state.kind === 'error') {
        const p = document.createElement('p')
        p.textContent = 'Error'
        body.append(p)
        return
      }

      // ok
      const list = document.createElement('ul')
      list.setAttribute('data-testid', 'deal-list')
      for (const deal of state.value.items) {
        const li = document.createElement('li')
        li.setAttribute('data-testid', `deal-${deal.id}`)
        li.textContent = `${deal.id} — ${deal.stage} — ${deal.amount}`
        list.append(li)
      }
      body.append(list)
    })

    effect(() => {
      mutationStatus.setAttribute('data-kind', close.state().kind)
    })
  })
}

mountQueryDemo()
