<!--
  Lab canvas for StatsGroup. Mount only — not a template.
  Source of truth: src/blocks/StatsGroup/StatsGroup.sail
-->
<scope>
  style canvas = { pad(6) }
</scope>

<column gap="3" style(canvas)>
  <small>StatsGroup</small>
  <row gap="4" align="stretch">
    <card>
      <strong>456,133</strong>
      <h3>Page views</h3>
      <p>24% more than in the same month last year, 33% more that two years ago</p>
    </card>
    <card>
      <strong>2,175</strong>
      <h3>New users</h3>
      <p>13% less compared to last month, new user engagement up by 6%</p>
    </card>
    <card>
      <strong>1,994</strong>
      <h3>Completed orders</h3>
      <p>1994 orders were completed this month, 97% satisfaction rate</p>
    </card>
  </row>
</column>
