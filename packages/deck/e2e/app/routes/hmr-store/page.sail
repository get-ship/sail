<controller>
import { hmrTally, storeBanner } from '@/stores/hmr-tally'
import { hmrLabel } from '@/compositions/hmr-label'

export default controller(() => {
  const { clicks, add } = useStore(hmrTally)
  const count = signal(0)
  // Plain getters (no `.set`) so restoreScope does not freeze the labels.
  const compositionLabel = () => hmrLabel
  const storeLabel = () => storeBanner
  return { clicks, add, count, compositionLabel, storeLabel }
})
</controller>

<div>
  <h1>HMR store</h1>
  <p data-testid="composition-label">{compositionLabel()}</p>
  <p data-testid="store-label">{storeLabel()}</p>
  <p>Page: <strong data-testid="page-count">{count()}</strong></p>
  <p>Store: <span data-testid="store-count">{clicks()}</span></p>
  <button on:click={() => count.set(count() + 1)}>inc-page</button>
  <button on:click={() => add()}>inc-store</button>
</div>
