<!--
  Lab canvas for FeaturesCards. Mount only — not a template.
  Source of truth: src/blocks/FeaturesCards/FeaturesCards.sail
-->
<scope>
  style canvas = { pad(6) }
</scope>

<column gap="3" style(canvas)>
  <small>FeaturesCards</small>
  <column gap="6">
    <column align="center" gap="3">
      <badge variant="solid" size="lg">Best company ever</badge>
      <h2>Integrate effortlessly with any technology stack</h2>
      <p>Every once in a while, you’ll see a Golbat that’s missing some fangs. This happens when hunger drives it to try biting a Steel-type Pokémon.</p>
    </column>
    <grid min="sm" gap="6">
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
          <icon name="check-circle" size="xl"></icon>
          <strong>No third parties</strong>
          <p>They’re popular, but they’re rare. Trainers who show them off recklessly may be targeted by thieves</p>
        </column>
      </card>
    </grid>
  </column>
</column>
