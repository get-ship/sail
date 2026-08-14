<!--
  Owned L1 rest-state strip. Theme paints tags. L1 owns host Style.
  Probe inventory comes from FAMILIES, not a hand list.
  Overlays that need a press stay off this page.
-->
<scope>
  style shell = { size({ min-h: fill }).pad(6).bg(background) }
  style measure = { size({ w: full, max-w: breakpoint-desktop }) }
</scope>

<column gap="8" style(shell)>
  <column gap="2" style(measure)>
    <row gap="3" align="center" justify="between">
      <column gap="2">
        <small>Owned L1 matrix</small>
        <h1>Rest state</h1>
      </column>
      <theme-picker />
    </row>
    <p>Not the docs site. Official themes only. Probe drives THEME_META; the picker is for eyes.</p>
  </column>

  <column gap="3">
    <h2>Intrinsic</h2>
    <p>No stretching parent — default measure.</p>
    <row gap="2" align="end">
      <button>Button</button>
      <badge>Badge</badge>
      <chip>Chip</chip>
      <input placeholder="input" />
      <a href="#">Link</a>
      <kbd>⌘K</kbd>
    </row>
  </column>

  <column gap="3" style(measure)>
    <h2>Actions</h2>
    <row gap="2">
      <button>Solid</button>
      <button variant="soft">Soft</button>
      <button variant="outline">Outline</button>
      <button variant="ghost">Ghost</button>
      <button tone="destructive">Destroy</button>
    </row>
    <row gap="2">
      <badge>rest</badge>
      <badge tone="primary">primary</badge>
      <chip>chip</chip>
    </row>
    <toggle-group value="day">
      <toggle value="day">Day</toggle>
      <toggle value="week">Week</toggle>
    </toggle-group>
    <menu-item value="edit">Edit</menu-item>
  </column>

  <column gap="3" style(measure)>
    <h2>Fields</h2>
    <field>
      <label slot="label">Name</label>
      <input placeholder="token, not hex" />
      <span slot="hint">Field + input.</span>
    </field>
    <file accept="image/*" />
    <textarea placeholder="notes"></textarea>
    <select>
      <option>One</option>
      <option>Two</option>
    </select>
    <date />
    <color />
    <slider value="40" />
    <meter value="0.4" />
    <row gap="4" align="center">
      <checkbox />
      <switch />
    </row>
    <radio-group name="plan" value="a">
      <radio value="a">A</radio>
      <radio value="b">B</radio>
    </radio-group>
  </column>

  <card>
    <h2 slot="header">Card</h2>
    <p>Surface, radius, elevation on the element.</p>
    <row slot="footer" gap="2">
      <button variant="ghost" size="sm">Cancel</button>
      <button size="sm">Save</button>
    </row>
  </card>

  <alert tone="info">
    <span slot="title">Alert</span>
    Theme tokens, not a painted div.
  </alert>

  <column gap="3" style(measure)>
    <h2>Families</h2>
    <p>Parts render inside the host recipe — not as peers.</p>
    <navbar bordered>
      <span slot="brand">Nav</span>
    </navbar>
    <breadcrumb>
      <a href="#">Home</a>
      <a href="#">Rest</a>
      <small>Here</small>
    </breadcrumb>
    <tabs active="one">
      <tab-list>
        <tab name="one">One</tab>
        <tab name="two">Two</tab>
      </tab-list>
      <tab-panel name="one">Panel one</tab-panel>
      <tab-panel name="two">Panel two</tab-panel>
    </tabs>
    <fieldset>
      <legend>Account</legend>
      <field>
        <label slot="label">Email</label>
        <input placeholder="you@ship.dev" />
      </field>
    </fieldset>
    <accordion>
      <span slot="trigger">Details</span>
      One disclosure.
    </accordion>
    <pagination value="1">
      <page value="1">1</page>
      <page value="2">2</page>
    </pagination>
    <table>
      <thead>
        <tr>
          <th>Col</th>
          <th>Val</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>A</td>
          <td>1</td>
        </tr>
      </tbody>
    </table>
  </column>

  <column gap="3" style(measure)>
    <h2>Copy</h2>
    <code-block code="const x = 1" lang="ts"></code-block>
    <prose>
      <p>Body with <strong>emphasis</strong>.</p>
    </prose>
    <ticker>
      <span>tokens · signals · rest</span>
    </ticker>
    <scroll-area>
      <p>Line one</p>
      <p>Line two</p>
    </scroll-area>
    <empty-state variant="inline">
      <span slot="title">Empty</span>
      No items.
    </empty-state>
    <error-state variant="inline">
      <span slot="title">Error</span>
      Try again.
    </error-state>
    <error-boundary>
      <p>Isolated.</p>
    </error-boundary>
  </column>

  <row gap="3" align="center" style(measure)>
    <avatar />
    <icon name="check" />
    <spinner />
    <skeleton />
    <progress value="40" />
    <kbd>⌘K</kbd>
    <separator />
  </row>
</column>
