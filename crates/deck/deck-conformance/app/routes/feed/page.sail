<controller>
import { controller, query } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ t: query(() => call('api.demo.clock', {})) }))
</controller>

<column gap="3">
  style.pad(6).maxW(breakpoint-tablet).mx(auto)

  <h1>
    style.fontSize(2xl).fontWeight(bold).fg(foreground)
Live Feed</h1>
  <while-loading {t}>
  <p>
    style.fg(muted-foreground).fontSize(sm)
Loading…</p>
  </while-loading>
  <while-error as="err" {t}>
  <p>
    style.fg(destructive).fontSize(sm)
Error: {err.message}</p>
  </while-error>
  <p>
    style.fg(muted-foreground).fontSize(sm).fontFamily(mono)
state: {t().kind}</p>
</column>
