<data name="agents" source="ship.agents.summaries()" live />

<section>
  {#while-loading of agents}
    <p data-test="loading">Loading agents...</p>
  {/while-loading}
  {#while-error err of agents}
    <p data-test="error">Error: {err.message}</p>
  {/while-error}
  {#if agents().kind === 'ok'}
    {#if agents().value.items.length === 0}
      <p data-test="empty">No agents.</p>
    {:else}
      <ul data-test="grid">
        {#for agent of agents().value.items key={agent.id}}
          <li data-test="agent">{agent.name}</li>
        {/for}
      </ul>
    {/if}
  {/if}
</section>
