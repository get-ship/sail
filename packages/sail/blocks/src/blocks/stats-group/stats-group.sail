<!--
  StatsGroup — a row of named figures. Defaults only.
  min=lg (24rem) is three 1fr tracks at 1280 and a stack at 390.
  cols=3 would keep three tracks on a phone and clip the figure.
  Number is the loud type; label (h3) and description (p) keep tag defaults.
-->
<scope>
  style figure = { type(display-sm) }
</scope>

<grid min="lg" gap="4">
  <card>
    <strong style(figure)>456,133</strong>
    <h3>Page views</h3>
    <p>24% more than in the same month last year, 33% more that two years ago</p>
  </card>
  <card>
    <strong style(figure)>2,175</strong>
    <h3>New users</h3>
    <p>13% less compared to last month, new user engagement up by 6%</p>
  </card>
  <card>
    <strong style(figure)>1,994</strong>
    <h3>Completed orders</h3>
    <p>1994 orders were completed this month, 97% satisfaction rate</p>
  </card>
</grid>
