<controller>
import { controller, signal } from '@ship/sail-runtime'

// A view WITH a controller can't be served from source — its compiled module
// imports `./controller`, so deck must produce a Rolldown bundle. The signal's
// initial value lives here (JS), so SSR can't know it: the server renders an
// empty reactive slot + hydration anchor, and the client fills `0` on mount.
// (Contrast `views/hello`, whose `{1 + 1}` is a constant the SSR pass folds.)
export default controller(() => ({ count: signal(0) }))
</controller>

<column gap="3" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <h1 sx="text-2xl font-bold text-foreground">Counter</h1>
  <p sx="text-muted-foreground text-sm">Count: <strong sx="text-foreground font-mono">{count()}</strong></p>
  <button on:click={() => count.set(count() + 1)}>increment</button>
</column>
