<!--
  CardWithStats — one card, a photo, a few figures.
    img alt: Running challenge
    title: Running challenge
    text: 80% completed
    text: 56 km this month • 17% improvement compared to last month • 443 place in global scoreboard
    stats: Distance 27.4 km | Avg. speed 9.6 km/h | Score 88/100
  size({ w: 38ch, max-w: 100% }) is the card, not a banner. Local photo
  — Unsplash never arrived. Cover fills the L1 image crop. Footer
  kickers are type(label).
-->
<scope>
  style frame = { size({ w: 38ch, max-w: 100% }) }
  style cover = { size(max).objectFit(cover) }
  style inset = { pad(5) }
  style kicker = { type(label).fg(muted) }
  style dek = { type(body-sm).fg(muted) }
</scope>

<card padding="none" style(frame)>
  <img
    slot="image"
    src="/running-challenge.jpg"
    alt="Running challenge"
    style(cover)
  >
  <column gap="3" style(inset)>
    <row nowrap justify="between" align="center" gap="2">
      <h3>Running challenge</h3>
      <badge variant="soft" size="sm">80% completed</badge>
    </row>
    <p style(dek)>56 km this month • 17% improvement compared to last month • 443 place in global scoreboard</p>
  </column>
  <row slot="footer" justify="between" gap="3" style(inset)>
    <column gap="1">
      <small style(kicker)>Distance</small>
      <strong>27.4 km</strong>
    </column>
    <column gap="1">
      <small style(kicker)>Avg. speed</small>
      <strong>9.6 km/h</strong>
    </column>
    <column gap="1">
      <small style(kicker)>Score</small>
      <strong>88/100</strong>
    </column>
  </row>
</card>
