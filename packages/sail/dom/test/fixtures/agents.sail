<section>
  <match {agents()}>
    {loading => <p data-test="loading">Loading agents...</p>}
    {error as err => <p data-test="error">Error: {err.message}</p>}
    {ok as value =>
      <if {value.items.length === 0}>
        <p data-test="empty">No agents.</p>
        {else =>
          <ul data-test="grid">
            <for {value.items as agent} key={agent.id}>
              <li data-test="agent">{agent.name}</li>
            </for>
          </ul>
        }
      </if>
    }
  </match>
</section>
