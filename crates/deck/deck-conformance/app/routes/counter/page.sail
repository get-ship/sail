<controller>
import { controller, signal } from '@ship/sail-runtime'

// A route WITH a controller can't be served from source — its compiled module
// imports `./controller`, so deck must produce a Rolldown bundle. The signal's
// initial value lives here (JS), so SSR can't know it: the server renders an
// empty reactive slot + hydration anchor, and the client fills `0` on mount.
// (Contrast `routes/hello`, whose `{1 + 1}` is a constant the SSR pass folds.)
export default controller(() => ({ count: signal(0) }))
</controller>

<column gap="3" style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })>
  <h1 style({ paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })>Counter</h1>
  <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>Count: <strong style({ paint: fg(foreground), text: fontFamily(mono) })>{count()}</strong></p>
  <button on:click={() => count.set(count() + 1)}>increment</button>
</column>
