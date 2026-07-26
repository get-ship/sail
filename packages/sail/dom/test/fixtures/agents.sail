<section>
  <while-loading of="agents">
    <p data-test="loading">Loading agents...</p>
  </while-loading>
  <while-error as="err" of="agents">
    <p data-test="error">Error: {err.message}</p>
  </while-error>
  <if cond={agents().kind === 'ok'}>
    <if cond={agents().value.items.length === 0}>
      <p data-test="empty">No agents.</p>
    <else>
      <ul data-test="grid">
        <for as="agent" of={agents().value.items} key={agent.id}>
          <li data-test="agent">{agent.name}</li>
        </for>
      </ul>
    </else></if>
  </if>
</section>
