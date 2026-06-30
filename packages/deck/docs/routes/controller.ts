// Overview controller — supplies the page's code snippets as strings (literal
// braces in a .sui template parse as interpolation, so code lives here).
import { controller, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

const quickstart = `my-app/
└─ routes/
   └─ page.sui        <h1>Hello, deck.</h1>

$ deck dev
  ▸ serving http://127.0.0.1:7878`

const layers = `SUI    :  deck   :  your app
React  :  Next   :  your app`

export default controller({
  scope: (_ctx: ViewScope<Scope>) => ({ quickstart, layers }),
})
