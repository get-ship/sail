<controller>
  export default controller(() => {
    const quotes = [
      {
        id: 'ada',
        quote:
          '“We dropped a custom design system and kept the tokens. Studio is the first theme tool the designers actually open.”',
        name: 'Ada L.',
        role: 'Design lead, Ada Labs',
        initials: 'AL',
      },
      {
        id: 'harbor',
        quote:
          '“The pricing page used to be a full-bleed accident. Defaults now hold a measure. That is the whole product.”',
        name: 'Harbor Co',
        role: 'Founder',
        initials: 'HC',
        presence: 'online',
      },
      {
        id: 'north',
        quote:
          '“Enterprise bought us a private catalog and an audit log. We did not rewrite a component to get there.”',
        name: 'North Pier',
        role: 'Platform',
        initials: 'NP',
        ring: true,
      },
    ]
    return { quotes }
  })
</controller>

<column gap="5">
  <h2>Loved by the people who theme it</h2>
  <grid min="sm" gap="4">
    <for {quotes as q} key={q.id}>
      <card variant="soft">
        <column gap="4">
          <p>{q.quote}</p>
          <row gap="3" align="center">
            <avatar initials={q.initials} presence={q.presence} ring={q.ring}></avatar>
            <column gap="0">
              <strong>{q.name}</strong>
              <small>{q.role}</small>
            </column>
          </row>
        </column>
      </card>
    </for>
  </grid>
</column>
