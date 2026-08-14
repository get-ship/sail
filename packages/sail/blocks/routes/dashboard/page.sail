<!--
  dashboard — first @ship/sail-blocks recipe.
  Defaults only. Host props (bordered, tone) are fine.
  Per-tag style() on L1 is a finding: defaults wrong or a primitive missing.
-->
<scope>
  style shell = { size({ min-h: fill }).bg(background) }
  style page = { pad(6) }
</scope>

<column gap="6" style(shell)>
  <navbar bordered>
    <span slot="brand">Northwind</span>
    <row slot="center" gap="2">
      <button variant="ghost" size="sm">Overview</button>
      <button variant="ghost" size="sm">Orders</button>
      <button variant="ghost" size="sm">Customers</button>
    </row>
    <button slot="actions" size="sm">New order</button>
  </navbar>

  <column gap="6" style(page)>
    <column gap="2">
      <h1>Overview</h1>
      <p>Defaults only — no per-tag paint.</p>
    </column>

    <row gap="4">
      <card>
        <small slot="header">Revenue</small>
        <h2>$48,200</h2>
        <span slot="footer">This month</span>
      </card>
      <card>
        <small slot="header">Orders</small>
        <h2>312</h2>
        <span slot="footer">Open + fulfilled</span>
      </card>
      <card>
        <small slot="header">Alerts</small>
        <h2>4</h2>
        <span slot="footer">Need review</span>
      </card>
    </row>

    <tabs active="orders">
      <tab-list>
        <tab name="orders">Orders</tab>
        <tab name="customers">Customers</tab>
      </tab-list>
      <tab-panel name="orders">
        <table>
          <thead>
            <tr>
              <th>Order</th>
              <th>Customer</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>#1042</td>
              <td>Harbor Co</td>
              <td>Open</td>
            </tr>
            <tr>
              <td>#1041</td>
              <td>Lee</td>
              <td>Fulfilled</td>
            </tr>
          </tbody>
        </table>
      </tab-panel>
      <tab-panel name="customers">
        <field>
          <label slot="label">Search</label>
          <input placeholder="Customer name" />
        </field>
      </tab-panel>
    </tabs>

    <alert tone="info">
      <span slot="title">Composition</span>
      Navbar, card, tabs, table, field, button — rest defaults.
    </alert>
  </column>
</column>
