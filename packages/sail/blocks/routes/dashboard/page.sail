<!--
  dashboard — Catalyst-shaped app chrome: rail, breadcrumb, KPIs, tabs,
  filters, sticky table, new-order dialog, toast. Measure is contained.
-->
<scope>
  style shell = { size({ min-h: fill }).bg(background) }
  style main = { grow(1).size({ min-w: 0 }).pad(6) }
  style measure = { size({ w: full, max-w: breakpoint-desktop }) }
  style kicker = { fg(muted-foreground).type({ size: sm }) }
  style queryCol = { grow(1).size({ min-w: 0, max-w: panel-max-w-sm }) }
  style kpi = { grow(1).size({ min-w: 0 }) }
  style metric = { type({ size: 3xl, weight: semibold, leading: none, nums: tabular }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
</scope>

<controller>
  export default controller(() => {
    const toast = signal(false)
    return {
      toast,
      created: (): void => toast.set(true),
    }
  })
</controller>

<row nowrap gap="0" align="stretch" style(shell)>
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

  <column gap="6" style(main)>
    <column gap="6" style(measure)>
      <alert variant="banner" tone="warning">
        <span slot="title">Four orders need a dock slot</span>
        Assign Harbor Co before 14:00 or they slip to tomorrow.
      </alert>

      <row justify="between" align="center">
        <column gap="2">
          <breadcrumb>
            <a href="/dashboard">Northwind</a>
            <small>Overview</small>
          </breadcrumb>
          <h1>Overview</h1>
          <p style(kicker)>Orders this week. Defaults only — if this looks wrong, the atoms are wrong.</p>
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
          <button command="close" commandfor="new-order" on:click={created}>Create order</button>
        </footer>
      </dialog>

      <row gap="4" align="stretch">
        <column style(kpi)>
          <card>
            <small slot="header">Revenue</small>
            <h2 style(metric)>$48,200</h2>
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
        <column style(kpi)>
          <card>
            <small slot="header">Open orders</small>
            <h2 style(metric)>18</h2>
            <span slot="footer" style(muted)>4 due today</span>
          </card>
        </column>
        <column style(kpi)>
          <card>
            <small slot="header">Fill rate</small>
            <h2 style(metric)>96%</h2>
            <meter slot="footer" value="96" min="0" max="100"></meter>
          </card>
        </column>
      </row>

      <tabs active="orders">
        <tab-list>
          <tab name="orders">Orders</tab>
          <tab name="customers">Customers</tab>
        </tab-list>

        <tab-panel name="orders">
          <card>
            <row slot="header" justify="between" align="center">
              <h2>Orders</h2>
              <badge tone="info">Live</badge>
            </row>
            <column gap="4">
              <row gap="3" align="end">
                <column style(queryCol)>
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
                  <span style(muted)>or</span>
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
                        <span>Harbor Co</span>
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
                        <span>Lee</span>
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
                        <span>North Pier</span>
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
                        <span>Ada Labs</span>
                      </row>
                    </td>
                    <td><badge tone="success">Fulfilled</badge></td>
                    <td>$640</td>
                    <td></td>
                  </tr>
                </tbody>
              </table>

              <pagination variant="simple">
                <page value="1">1</page>
                <page value="2">2</page>
                <page value="3">3</page>
              </pagination>
            </column>
          </card>
        </tab-panel>

        <tab-panel name="customers">
          <empty-state compact>
            <icon slot="icon" name="inbox"></icon>
            <span slot="title">No customer selected</span>
            Pick an order, or create one with New order.
            <button slot="action" command="show-modal" commandfor="new-order">New order</button>
          </empty-state>
        </tab-panel>
      </tabs>
    </column>
  </column>
</row>

<if {toast()}>
  <toast-region position="bottom-end">
    <toast tone="success">Order created for Harbor Co.</toast>
  </toast-region>
</if>
