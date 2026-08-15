<!--
  NavbarMinimal — icon rail. Defaults only. Not app chrome for a whole product.
  Head-to-head: https://ui.mantine.dev/component/navbar-minimal/
  Recorded from Mantine UnstyledButton aria-label / tooltip:
    Home, Dashboard, Analytics (active), Releases, Account, Security, Settings
    Change account, Logout
  Icons: CORE_ROLES names only.
-->
<navbar placement="rail" bordered>
  <icon slot="brand" name="ship"></icon>
  <column slot="center" gap="1" align="center">
    <button variant="ghost" size="icon" label="Home">
      <icon name="home"></icon>
    </button>
    <button variant="ghost" size="icon" label="Dashboard">
      <icon name="activity"></icon>
    </button>
    <button variant="soft" size="icon" label="Analytics" current="true">
      <icon name="compass"></icon>
    </button>
    <button variant="ghost" size="icon" label="Releases">
      <icon name="rocket"></icon>
    </button>
    <button variant="ghost" size="icon" label="Account">
      <icon name="user"></icon>
    </button>
    <button variant="ghost" size="icon" label="Security">
      <icon name="warning"></icon>
    </button>
    <button variant="ghost" size="icon" label="Settings">
      <icon name="settings"></icon>
    </button>
  </column>
  <column slot="actions" gap="1" align="center">
    <button variant="ghost" size="icon" label="Change account">
      <icon name="refresh"></icon>
    </button>
    <button variant="ghost" size="icon" label="Logout">
      <icon name="log-out"></icon>
    </button>
  </column>
</navbar>
