<controller>
import { controller, query } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ t: query(() => call('api.demo.clock', {})) }))
</controller>

<column gap="3" style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })>
  <h1 style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })>Live Clock</h1>
  <match {t()}>
  {loading => <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>Loading…</p>}
  {error as err => <p style({ paint: fg(destructive), text: fontSize(sm) })>Error: {err.message}</p>}
  {ok as value => <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>Tick: <strong style({ paint: fg(foreground), text: fontFamily(mono) })>{value.now}</strong></p>}
</match>
</column>
