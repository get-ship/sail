<!--
  dashboard — Catalyst-shaped app chrome: rail, breadcrumb, KPIs, tabs,
  filters, sticky table, new-order dialog, toast. Measure is contained.
-->


<controller>
  export default controller(() => {
    const orderCreated = signal(false)
    return {
      orderCreated,
      created: (): void => orderCreated.set(true),
    }
  })
</controller>

<row nowrap gap="0" align="stretch">
  <navbar placement="rail" bordered sticky>
    <strong slot="brand">Northwind</strong>
    <column slot="center" gap="1">
      <button variant="ghost" size="sm">Overview</button>
      <button variant="ghost" size="sm">Orders</button>
      <button variant="ghost" size="sm">Customers</button>
      <a href="/settings">Settings</a>
    </column>
    <column slot="actions" gap="3" align="center">
      <theme-picker></theme-picker>
      <avatar initials="NW" tone="primary" presence="online"></avatar>
    </column>
  </navbar>

  <column gap="6">
    <column gap="6">
      <alert dismissible tone="warning">
        <strong slot="title">Four orders need a dock slot</strong>
        Assign Harbor Co before 14:00 or they slip to tomorrow.
      </alert>

      <row justify="between" align="center">
        <column gap="2">
          <breadcrumb>
            <a href="/dashboard">Northwind</a>
            <small>Overview</small>
          </breadcrumb>
          <h1>Overview</h1>
          <p>Orders this week. Defaults only — if this looks wrong, the atoms are wrong.</p>
        </column>
        <row gap="2" align="center">
          <row>
            <avatar initials="HC" stack ring></avatar>
            <avatar initials="AL" stack ring></avatar>
            <avatar initials="NP" stack></avatar>
          </row>
          <button variant="outline" command="show" commandfor="order-filters">Filters</button>
          <button command="show-modal" commandfor="new-order">New order</button>
        </row>
      </row>

      <drawer id="order-filters" side="end" size="sm">
        <header>Filters</header>
        <section>
          <column gap="4">
            <field>
              <label slot="label">Customer</label>
              <input placeholder="Harbor Co" />
            </field>
            <field>
              <label slot="label">Needed by</label>
              <date />
            </field>
            <column gap="2">
              <label>Status</label>
              <label><checkbox /> Open</label>
              <label><checkbox /> Fulfilled</label>
              <label><checkbox /> Held</label>
            </column>
          </column>
        </section>
        <footer>
          <button command="close" commandfor="order-filters" variant="ghost">Clear</button>
          <button command="close" commandfor="order-filters">Apply</button>
        </footer>
      </drawer>

      <dialog id="new-order">
        <header>New order</header>
        <section>
          <form label="New order">
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
          </form>
        </section>
        <footer>
          <button command="close" commandfor="new-order" variant="ghost">Cancel</button>
          <button command="close" commandfor="new-order" on:click={created}>Create order</button>
        </footer>
      </dialog>

      <grid min="sm" gap="4">
        <column>
          <card>
            <small slot="header">Revenue</small>
            <h2>$48,200</h2>
            <row slot="footer" gap="2" align="center">
              <badge tone="success" dot>Live</badge>
              <a href="#" popovertarget="rev-tip">+12% vs last week</a>
            </row>
          </card>
          <hover-card id="rev-tip">
            <strong>Week over week</strong>
            <p>Revenue versus the previous seven days.</p>
          </hover-card>
        </column>
        <card>
          <small slot="header">Open orders</small>
          <h2>18</h2>
          <small slot="footer">4 due today</small>
        </card>
        <card>
          <small slot="header">Fill rate</small>
          <h2>96%</h2>
          <meter slot="footer" value="96" min="0" max="100"></meter>
        </card>
      </grid>

      <tabs active="orders">
        <tab-list>
          <tab title="Orders" name="orders"></tab>
          <tab title="Customers" name="customers"></tab>
        </tab-list>

        <tab-panel name="orders">
          <card>
            <row slot="header" justify="between" align="center">
              <h2>Orders</h2>
              <badge tone="info">Live</badge>
            </row>
            <column gap="4">
              <row gap="3" align="end">
                <column>
                  <field>
                    <label slot="label">Search</label>
                    <combobox placeholder="Order or customer">
                      <option value="1042">#1042 Harbor Co</option>
                      <option value="1041">#1041 Lee</option>
                      <option value="1040">#1040 North Pier</option>
                      <option value="1039">#1039 Ada Labs</option>
                    </combobox>
                  </field>
                </column>
                <field>
                  <label slot="label">Status</label>
                  <select>
                    <option>All</option>
                    <option>Open</option>
                    <option>Fulfilled</option>
                    <option>Held</option>
                  </select>
                </field>
                <row gap="1" align="center">
                  <small>or</small>
                  <kbd>⌘</kbd>
                  <kbd>K</kbd>
                </row>
              </row>

              <row gap="2">
                <chip selected>All</chip>
                <chip variant="outline">Open</chip>
                <chip variant="outline">Fulfilled</chip>
                <chip variant="outline">Held</chip>
              </row>

              <table sticky-header hoverable>
                <thead>
                  <tr>
                    <th>Order</th>
                    <th>Customer</th>
                    <th>Status</th>
                    <th>Total</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  <tr>
                    <td>#1042</td>
                    <td>
                      <row gap="2" align="center">
                        <avatar initials="HC" size="xs"></avatar>
                        Harbor Co
                      </row>
                    </td>
                    <td><badge tone="warning">Open</badge></td>
                    <td>$1,240</td>
                    <td>
                      <button size="sm" variant="ghost" popovertarget="row-1042">Actions</button>
                      <dropdown id="row-1042" placement="bottom-end">
                        <menu-item value="open">Open</menu-item>
                        <menu-item value="hold">Hold</menu-item>
                        <menu-item value="cancel" tone="destructive">Cancel</menu-item>
                      </dropdown>
                    </td>
                  </tr>
                  <tr>
                    <td>#1041</td>
                    <td>
                      <row gap="2" align="center">
                        <avatar initials="LE" size="xs"></avatar>
                        Lee
                      </row>
                    </td>
                    <td><badge tone="success">Fulfilled</badge></td>
                    <td>$380</td>
                    <td></td>
                  </tr>
                  <tr>
                    <td>#1040</td>
                    <td>
                      <row gap="2" align="center">
                        <avatar initials="NP" size="xs"></avatar>
                        North Pier
                      </row>
                    </td>
                    <td><badge>Held</badge></td>
                    <td>$2,010</td>
                    <td></td>
                  </tr>
                  <tr>
                    <td>#1039</td>
                    <td>
                      <row gap="2" align="center">
                        <avatar initials="AL" size="xs"></avatar>
                        Ada Labs
                      </row>
                    </td>
                    <td><badge tone="success">Fulfilled</badge></td>
                    <td>$640</td>
                    <td></td>
                  </tr>
                </tbody>
              </table>

              <pagination variant="simple">
                <page-item value="1">1</page-item>
                <page-item value="2">2</page-item>
                <page-item value="3">3</page-item>
              </pagination>
            </column>
          </card>
        </tab-panel>

        <tab-panel name="customers">
          <empty-state compact>
            <icon slot="icon" name="inbox"></icon>
            <strong slot="title">No customer selected</strong>
            Pick an order, or create one with New order.
            <button slot="action" command="show-modal" commandfor="new-order">New order</button>
          </empty-state>
        </tab-panel>
      </tabs>
    </column>
  </column>
</row>

<if {orderCreated()}>
  <toast-region position="bottom-end">
    <toast tone="success">Order created for Harbor Co.</toast>
  </toast-region>
</if>
