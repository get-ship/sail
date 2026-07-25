<controller>
import { controller, query } from '@ship/sail-runtime'
import { call } from 'ship'
export default controller((_ctx) => ({ result: query(() => call('ship.demo.boom', {})) }))
</controller>

<column gap="3" sx={style.pad(6).maxW(breakpoint-tablet).mx(auto)}>
  <h1 sx={style.fontSize(2xl).fontWeight(bold).fg(foreground)}>Error Boundary</h1>
  {#while-loading of result}
  <p sx={style.fg(muted-foreground).fontSize(sm)}>Loading…</p>
  {/while-loading}
  {#while-error err of result}
  <p sx={style.fg(muted-foreground).fontSize(sm)}>Something went wrong</p>
  {/while-error}
  {#if result().kind === 'ok'}
  <p sx={style.fg(muted-foreground).fontSize(sm)}>Value: <strong sx={style.fg(foreground).fontFamily(mono)}>{result().value.data}</strong></p>
  {/if}
</column>
