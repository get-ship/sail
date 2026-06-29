// Recipes index controller. Reads the LOCAL data client off the scope-factory
// context (`ctx.clients.data`) and exposes its reactive `recipes` signal to the
// template. Because the client is a shared singleton, adding a recipe in the
// kitchen re-renders this list with no extra wiring.
//
// Canonical controller shape: object form + a `scope` factory typed with the
// compiler-generated `ViewScope<Scope>`, so the template's `recipes()` reads
// are statically discovered (the compiler extracts scope from this returned
// object literal).
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller({
  scope: (ctx: ViewScope<Scope>) => ({
    recipes: ctx.clients.data.recipes,
  }),
})
