<controller>
// /push-demo — browser-probe fixture for query()'s PUSH path. `feed` is an
// ES-imported client; `feed.clock.stream()` returns a LiveSource<Tick>, so
// query() subscribes (push) instead of fetching once (pull) — the SAME
// query() call, routed by the method's return-value shape. Each interval
// tick re-renders the tick count below with zero console errors.

import { query } from '@ship/sail-runtime'
import { feed } from './push-clock'

export default controller(() => {
  const tick = query(() => feed.clock.stream())
  return { tick }
})
</controller>

<column gap="3" sx={style.pad(6).maxW(breakpoint-tablet).mx(auto)}>
  <h1 sx={style.fontSize(2xl).fontWeight(bold).fg(foreground)}>push() demo</h1>
  <p sx={style.fg(muted-foreground).fontSize(sm)}>
    A client method returning <code>LiveSource&lt;T&gt;</code> makes
    <code>query()</code> subscribe — one <code>ok</code> per push.
  </p>

  <if cond={tick().kind === 'loading'}>
  <p sx={style.fg(muted-foreground).fontSize(sm)} data-testid="push-loading">waiting for first push…</p>
  </if>
  <if cond={tick().kind === 'error'}>
  <p sx={style.fg(destructive).fontSize(sm)} data-testid="push-error">Error</p>
  </if>
  <if cond={tick().kind === 'ok'}>
  <p sx={style.fg(foreground).fontFamily(mono)} data-testid="push-tick">tick #{tick().value.n}</p>
  <p sx={style.fg(muted-foreground).fontSize(xs).fontFamily(mono)} data-testid="push-at">{tick().value.at}</p>
  </if>
</column>
