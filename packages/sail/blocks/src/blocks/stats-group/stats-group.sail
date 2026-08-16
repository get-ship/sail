<!--
  StatsGroup — one band, three named figures. Defaults only.
  One card, not three leftover SaaS cards. min=lg stacks at 390.
  Number is display-sm. Title is type(label) so it is a kicker, not an
  h3 heading/xl twin. Dek is body-sm muted. No gradient.
-->
<scope>
  style figure = { type(display-sm) }
  style kicker = { type(label).fg(muted) }
  style dek = { type(body-sm).fg(muted) }
</scope>

<card padding="lg">
  <grid min="lg" gap="8">
    <column gap="1">
      <strong style(figure)>456,133</strong>
      <h3 style(kicker)>Page views</h3>
      <p style(dek)>24% more than in the same month last year, 33% more that two years ago</p>
    </column>
    <column gap="1">
      <strong style(figure)>2,175</strong>
      <h3 style(kicker)>New users</h3>
      <p style(dek)>13% less compared to last month, new user engagement up by 6%</p>
    </column>
    <column gap="1">
      <strong style(figure)>1,994</strong>
      <h3 style(kicker)>Completed orders</h3>
      <p style(dek)>1994 orders were completed this month, 97% satisfaction rate</p>
    </column>
  </grid>
</card>
