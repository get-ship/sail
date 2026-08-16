<controller>
// Throws from an effect on mount with no error boundary — proves the deck
// dev overlay renders for the no-boundary path.
export default controller(() => {
  effect(() => {
    throw new Error('deliberate boom for overlay e2e')
  })
  return {}
})
</controller>

<div>
  <h1>error boom</h1>
  <a href="/counter">counter</a>
</div>
