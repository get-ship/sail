<controller>
import { controller, query } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ result: query(() => call('ship.demo.boom', {})) }))
</controller>

<column gap="3">
  style.pad(6).maxW(breakpoint-tablet).mx(auto)

  <h1>
    style.fontSize(2xl).fontWeight(bold).fg(foreground)
Error Boundary</h1>
  <while-loading {result}>
  <p>
    style.fg(muted-foreground).fontSize(sm)
Loading…</p>
  </while-loading>
  <while-error as="err" {result}>
  <p>
    style.fg(muted-foreground).fontSize(sm)
Something went wrong</p>
  </while-error>
  <if {result().kind === 'ok'}>
  <p>
    style.fg(muted-foreground).fontSize(sm)
Value: <strong>
    style.fg(foreground).fontFamily(mono)
{result().value.data}</strong></p>
  </if>
</column>
