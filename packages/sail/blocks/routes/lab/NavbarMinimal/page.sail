<!--
  Lab canvas for NavbarMinimal. Mount only — not a template.
  Source of truth: src/blocks/NavbarMinimal/NavbarMinimal.sail
-->
<scope>
  style canvas = { pad(6) }
</scope>

<column gap="3" style(canvas)>
  <small>NavbarMinimal</small>
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
</column>
