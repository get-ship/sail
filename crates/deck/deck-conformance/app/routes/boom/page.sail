<controller>
import { controller, query } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ result: query(() => call('ship.demo.boom', {})) }))
</controller>

<column gap="3">
  style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })

  <h1>
    style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })
Error Boundary</h1>
  <match {result()}>
  {loading => <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm) })
Loading…</p>}
  {error as err => <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm) })
Something went wrong</p>}
  {ok as value => <p>
    style({ paint: fg(muted-foreground), text: fontSize(sm) })
Value: <strong>
    style({ paint: fg(foreground), text: fontFamily(mono) })
{value.data}</strong></p>}
</match>
</column>
