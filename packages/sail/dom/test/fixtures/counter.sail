<div>
  <p>count: <span data-test="value">{count()}</span></p>
  <button data-test="inc" on:click={() => count.set(count() + 1)}>
    increment
  </button>
  <button data-test="reset" on:click={() => count.set(0)}>
    reset
  </button>
</div>
