<!--
  app-rail — Northwind chrome. Collapsed is NavbarMinimal (icon rail).
  Expanded is a labeled menu at panel.rail-w. Not a new L1 tag.
-->

<scope>
  props: {
    current: 'overview' | 'orders' | 'customers' | 'settings' = 'overview'
  }

  style show = {display(inline)}
  style lab = {display(none).type(body-sm), when: { group-expanded: show }}
</scope>

<controller>
  export default controller(() => {
    const expanded = signal(true)
    return {
      expanded,
      toggle: (): void => expanded.set(!expanded()),
    }
  })
</controller>

<navbar placement="rail" bordered sticky expanded={expanded()}>
  <a slot="brand" href="/dashboard" variant="ghost">
    <row nowrap align="center" gap="2">
      <icon name="ship"></icon>
      <small style(lab)>Northwind</small>
    </row>
  </a>
  <column slot="center" gap="1">
    <a href="/dashboard" variant={current() === 'overview' ? 'soft' : 'ghost'}>
      <row nowrap align="center" gap="2">
        <icon name="home"></icon>
        <small style(lab)>Overview</small>
      </row>
    </a>
    <a href="/dashboard" variant={current() === 'orders' ? 'soft' : 'ghost'}>
      <row nowrap align="center" gap="2">
        <icon name="activity"></icon>
        <small style(lab)>Orders</small>
      </row>
    </a>
    <a href="/dashboard" variant={current() === 'customers' ? 'soft' : 'ghost'}>
      <row nowrap align="center" gap="2">
        <icon name="user"></icon>
        <small style(lab)>Customers</small>
      </row>
    </a>
    <a href="/settings" variant={current() === 'settings' ? 'soft' : 'ghost'}>
      <row nowrap align="center" gap="2">
        <icon name="settings"></icon>
        <small style(lab)>Settings</small>
      </row>
    </a>
  </column>
  <column slot="actions" gap="2" align="center">
    <theme-picker></theme-picker>
    <avatar initials="NW" tone="primary" presence="online"></avatar>
    <button variant="ghost" size="icon" label={expanded() ? 'Collapse menu' : 'Expand menu'} on:click={toggle}>
      <icon name={expanded() ? 'chevron-left' : 'chevron-right'}></icon>
    </button>
  </column>
</navbar>
