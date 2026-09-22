<!-- Owned tutorial fixture: state lives in <controller>; the template
     binds only what the default export returns. -->
<controller>
export default controller(() => {
  const count = signal(0)
  return {
    count,
    inc: () => count.set(count() + 1),
  }
})
</controller>

<button on:click={() => inc()}>Clicked {count()}</button>
