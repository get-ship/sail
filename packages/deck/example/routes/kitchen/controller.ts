// Kitchen controller — the interactivity + mutation showcase.
//
//  • `count`    — the canonical signal counter (`count.set(count() + 1)`).
//  • `timerOn`  — a boolean toggle.
//  • `title` / `minutes` — form inputs, two-way bound via `bind:value`.
//  • `addRecipe` — the mutation: appends to the shared data client, then
//    navigates to the new recipe's dynamic route.
import { controller, signal, type ViewScope } from '@ship/sui-runtime'
import { navigate } from '@ship/deck/router'
import type { Scope } from './page.sui'

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const count = signal(0)
    const timerOn = signal(false)
    const title = signal('')
    const minutes = signal('15')

    const addRecipe = (): void => {
      const t = title().trim()
      if (!t) return
      const created = ctx.clients.data.add({ title: t, minutes: Number(minutes()) || 0 })
      title.set('')
      minutes.set('15')
      navigate(`/recipes/${created.id}`)
    }

    return {
      count,
      timerOn,
      title,
      minutes,
      recipes: ctx.clients.data.recipes,
      addRecipe,
    }
  },
})
