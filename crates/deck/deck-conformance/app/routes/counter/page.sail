<controller>
import { controller, signal } from '@ship/sail-runtime'

// A route WITH a controller can't be served from source — its compiled module
// imports `./controller`, so deck must produce a Rolldown bundle. The signal's
// initial value lives here (JS), so SSR can't know it: the server renders an
// empty reactive slot + hydration anchor, and the client fills `0` on mount.
// (Contrast `routes/hello`, whose `{1 + 1}` is a constant the SSR pass folds.)
export default controller(() => ({ count: signal(0) }))
</controller>

<column gap="3" sx={style.pad(6).maxW(breakpoint-tablet).mx(auto)}>
  <h1 sx={style.fontSize(2xl).fontWeight(bold).fg(foreground)}>Counter</h1>
  <p sx={style.fg(muted-foreground).fontSize(sm)}>Count: <strong sx={style.fg(foreground).fontFamily(mono)}>{count()}</strong></p>
  <button on:click={() => count.set(count() + 1)}>increment</button>
</column>
