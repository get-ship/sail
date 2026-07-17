<controller>
import { controller, live } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ t: live(() => call('api.demo.clock', {})) }))
</controller>

<column gap="3" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <h1 sx="text-2xl font-bold text-foreground">Live Feed</h1>
  {#while-loading of t}
  <p sx="text-muted-foreground text-sm">Loading…</p>
  {/while-loading}
  {#while-error err of t}
  <p sx="text-destructive text-sm">Error: {err.message}</p>
  {/while-error}
  <p sx="text-muted-foreground text-sm font-mono">state: {t().kind}</p>
</column>
