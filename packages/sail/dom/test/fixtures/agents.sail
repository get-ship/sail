<section>
  <while-loading {agents}>
    <p data-test="loading">Loading agents...</p>
  </while-loading>
  <while-error as="err" {agents}>
    <p data-test="error">Error: {err.message}</p>
  </while-error>
  <if {agents().kind === 'ok'}>
    <if {agents().value.items.length === 0}>
      <p data-test="empty">No agents.</p>
    <else>
      <ul data-test="grid">
        <for {agents().value.items} as="agent" key={agent.id}>
          <li data-test="agent">{agent.name}</li>
        </for>
      </ul>
    </else></if>
  </if>
</section>
