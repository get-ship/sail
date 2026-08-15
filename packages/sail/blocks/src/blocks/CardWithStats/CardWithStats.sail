<!--
  CardWithStats — one card, a few figures. Defaults only.
  Head-to-head: https://ui.mantine.dev/component/card-with-stats/
  Recorded from the Mantine page (no buttons, links, or fields):
    img alt: Running challenge
    title: Running challenge
    text: 80% completed
    text: 56 km this month • 17% improvement compared to last month • 443 place in global scoreboard
    stats: Distance 27.4 km | Avg. speed 9.6 km/h | Score 88/100
-->
<card padding="lg">
  <img
    slot="image"
    src="https://images.unsplash.com/photo-1581889470536-467bdbe30cd0?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=720&q=80"
    alt="Running challenge"
  />
  <row justify="between" align="center" gap="2">
    <h3>Running challenge</h3>
    <text>80% completed</text>
  </row>
  <text>56 km this month • 17% improvement compared to last month • 443 place in global scoreboard</text>
  <row slot="footer" justify="between" gap="3">
    <column>
      <text>Distance</text>
      <text>27.4 km</text>
    </column>
    <column>
      <text>Avg. speed</text>
      <text>9.6 km/h</text>
    </column>
    <column>
      <text>Score</text>
      <text>88/100</text>
    </column>
  </row>
</card>
