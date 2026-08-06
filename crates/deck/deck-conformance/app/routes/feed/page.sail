<controller>
import { controller, query } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ t: query(() => call('api.demo.clock', {})) }))
</controller>

<column gap="3">
  style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })

  <h1>
    style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })
Live Feed</h1>
  <while-loading {t}>
  <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm) })
Loading…</p>
  </while-loading>
  <while-error as="err" {t}>
  <p>
    style({ paint: fg(destructive), text: fontSize(sm) })
Error: {err.message}</p>
  </while-error>
  <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm).fontFamily(mono) })
state: {t().kind}</p>
</column>
