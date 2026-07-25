<controller>
import { controller, query } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ t: query(() => call('api.demo.clock', {})) }))
</controller>

<column gap="3" sx={style.pad(6).maxW(breakpoint-tablet).mx(auto)}>
  <h1 sx={style.fontSize(2xl).fontWeight(bold).fg(foreground)}>Live Clock</h1>
  {#while-loading of t}
  <p sx={style.fg(muted-foreground).fontSize(sm)}>Loading…</p>
  {/while-loading}
  {#while-error err of t}
  <p sx={style.fg(destructive).fontSize(sm)}>Error: {err.message}</p>
  {/while-error}
  {#if t().kind === 'ok'}
  <p sx={style.fg(muted-foreground).fontSize(sm)}>Tick: <strong sx={style.fg(foreground).fontFamily(mono)}>{t().value.now}</strong></p>
  {/if}
</column>
