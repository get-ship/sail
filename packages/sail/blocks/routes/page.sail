<!--
  Index — templates (pages) and authored blocks (named compositions).
  Not a marketplace. Catalog is the list.
-->


<controller>
import { blocks } from '../src/compositions/catalog'

export default controller(() => {
  const authored = signal(blocks().filter((b) => b.status === 'authored'))
  return { authored }
})
</controller>

<column gap="0">
  <navbar bordered>
    <strong slot="brand">Blocks</strong>
    <row slot="center" gap="2">
      <a href="/dashboard">Dashboard</a>
      <a href="/pricing">Pricing</a>
      <a href="/settings">Settings</a>
    </row>
    <theme-picker slot="actions"></theme-picker>
  </navbar>

  <page>
    <column gap="2">
      <row>
        <badge>Taste</badge>
      </row>
      <h1>Blocks</h1>
      <p>Templates are pages. Blocks are named compositions of L1 defaults. If either needs paint on every tag, the atoms are wrong.</p>
    </column>

    <column gap="3">
      <h2>Templates</h2>
      <grid min="lg" gap="4">
        <a href="/dashboard">
          <card interactive>
            <small slot="header">Template</small>
            <h3>Dashboard</h3>
            <p>Rail, dialog, filters, table, toast. The new-order path is the form default.</p>
          </card>
        </a>
        <a href="/pricing">
          <card interactive>
            <small slot="header">Template</small>
            <h3>Pricing</h3>
            <p>Three tiers, comparison table, quotes, FAQ. The Tailwind Plus bar.</p>
          </card>
        </a>
        <a href="/settings">
          <card interactive>
            <small slot="header">Template</small>
            <h3>Settings</h3>
            <p>Tabbed form. Stretch stops at the field column.</p>
          </card>
        </a>
      </grid>
    </column>

    <column gap="3">
      <h2>Blocks</h2>
      <p>Authored compositions. Canvas only — no page chrome.</p>
      <grid min="sm" gap="4">
        <for {authored() as b} key={b.id}>
          <a href={`/lab/${b.id}`}>
            <card interactive>
              <small slot="header">Block</small>
              <h3>{b.title}</h3>
              <p>{b.intent}</p>
            </card>
          </a>
        </for>
      </grid>
    </column>
  </page>
</column>
