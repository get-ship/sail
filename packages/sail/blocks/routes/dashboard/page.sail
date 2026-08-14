<!--
  dashboard — app chrome with rail, filters, table, and a new-order dialog.
  Page measure is contained. L1 hosts use props only.
-->
<scope>
  style shell = { size({ min-h: fill }).bg(background) }
  style main = { size({ min-w: 0 }).pad(6) }
  style kpis = { size({ w: full }) }
</scope>

<row gap="0" align="stretch" style(shell)>
  <navbar placement="rail" bordered sticky>
    <strong slot="brand">Northwind</strong>
    <column slot="center" gap="1">
      <button variant="ghost" size="sm">Overview</button>
      <button variant="ghost" size="sm">Orders</button>
      <button variant="ghost" size="sm">Customers</button>
      <button variant="ghost" size="sm">Settings</button>
    </column>
    <theme-picker slot="actions"></theme-picker>
  </navbar>

  <column gap="6" style(main)>
    <row justify="between" align="center">
      <column gap="1">
        <h1>Overview</h1>
        <p>Orders this week. Defaults only — if this looks wrong, the atoms are wrong.</p>
      </column>
      <button command="show-modal" commandfor="new-order">New order</button>
    </row>

    <dialog id="new-order">
      <header>New order</header>
      <section>
        <column gap="3">
          <field>
            <label slot="label">Customer</label>
            <input placeholder="Harbor Co" />
          </field>
          <field>
            <label slot="label">SKU</label>
            <select>
              <option>Canvas / 24×36</option>
              <option>Print / A2</option>
              <option>Kit / starter</option>
            </select>
          </field>
          <row gap="3">
            <field>
              <label slot="label">Qty</label>
              <input type="number" placeholder="12" />
            </field>
            <field>
              <label slot="label">Needed by</label>
              <date />
            </field>
          </row>
          <field>
            <label slot="label">Notes</label>
            <textarea placeholder="Dock 3, morning window"></textarea>
          </field>
        </column>
      </section>
      <footer>
        <button command="close" commandfor="new-order" variant="ghost">Cancel</button>
        <button command="close" commandfor="new-order">Create order</button>
      </footer>
    </dialog>

    <row gap="4" style(kpis)>
      <card>
        <small slot="header">Revenue</small>
        <h2>$48,200</h2>
        <span slot="footer">+12% vs last week</span>
      </card>
      <card>
        <small slot="header">Open orders</small>
        <h2>18</h2>
        <span slot="footer">4 due today</span>
      </card>
      <card>
        <small slot="header">Fill rate</small>
        <h2>96%</h2>
        <progress value="96" max="100"></progress>
      </card>
    </row>

    <card>
      <row slot="header" justify="between" align="center">
        <h2>Orders</h2>
        <badge tone="info">Live</badge>
      </row>
      <column gap="4">
        <row gap="3" align="end">
          <field>
            <label slot="label">Search</label>
            <input placeholder="Order or customer" />
          </field>
          <field>
            <label slot="label">Status</label>
            <select>
              <option>All</option>
              <option>Open</option>
              <option>Fulfilled</option>
              <option>Held</option>
            </select>
          </field>
          <button variant="outline" size="sm">Filter</button>
        </row>

        <table>
          <thead>
            <tr>
              <th>Order</th>
              <th>Customer</th>
              <th>Status</th>
              <th>Total</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>#1042</td>
              <td>Harbor Co</td>
              <td><badge tone="warning">Open</badge></td>
              <td>$1,240</td>
            </tr>
            <tr>
              <td>#1041</td>
              <td>Lee</td>
              <td><badge tone="success">Fulfilled</badge></td>
              <td>$380</td>
            </tr>
            <tr>
              <td>#1040</td>
              <td>North Pier</td>
              <td><badge>Held</badge></td>
              <td>$2,010</td>
            </tr>
            <tr>
              <td>#1039</td>
              <td>Ada Labs</td>
              <td><badge tone="success">Fulfilled</badge></td>
              <td>$640</td>
            </tr>
          </tbody>
        </table>
        <pagination></pagination>
      </column>
    </card>

    <alert tone="info">
      <span slot="title">Four orders need a dock slot</span>
      Assign Harbor Co before 14:00 or they slip to tomorrow.
    </alert>
  </column>
</row>
