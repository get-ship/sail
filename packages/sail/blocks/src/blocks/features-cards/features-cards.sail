<!--
  FeaturesCards — badge, heading, lead, three feature cards.
  Roles: heading "Integrate effortlessly with any technology stack".
  No buttons, links, or fields. Card titles are text, not headings.
  Heading cluster is <center>. Cards are an inline grid so they hug
  (minmax sm, not 1fr-to-the-glass) and the outer center sits the
  cluster in the middle. min=lg on a block grid is a fullscreen band.
-->
<column gap="6">
  <center gap="3">
    <badge variant="solid" size="lg">Best company ever</badge>
    <h2>Integrate effortlessly with any technology stack</h2>
    <p>Every once in a while, you’ll see a Golbat that’s missing some fangs. This happens when hunger drives it to try biting a Steel-type Pokémon.</p>
  </center>
  <center>
    <grid inline min="sm" gap="6">
      <card padding="lg">
        <column gap="2">
          <icon name="zap" size="xl"></icon>
          <strong>Extreme performance</strong>
          <p>This dust is actually a powerful poison that will even make a pro wrestler sick, Regice cloaks itself with frigid air of -328 degrees Fahrenheit</p>
        </column>
      </card>
      <card padding="lg">
        <column gap="2">
          <icon name="user" size="xl"></icon>
          <strong>Privacy focused</strong>
          <p>People say it can run at the same speed as lightning striking, Its icy body is so cold, it will not melt even if it is immersed in magma</p>
        </column>
      </card>
      <card padding="lg">
        <column gap="2">
          <icon name="success" size="xl"></icon>
          <strong>No third parties</strong>
          <p>They’re popular, but they’re rare. Trainers who show them off recklessly may be targeted by thieves</p>
        </column>
      </card>
    </grid>
  </center>
</column>
