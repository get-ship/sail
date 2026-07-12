// Snippets for the introduction. Every SUI sample is compiled by the build.
export const code = {
  install: `npm install @ship/sui-runtime`,

  page: `<!-- a page. That's it — no config, no providers, no CSS file. -->
<column gap="4" sx="p:6 max-w:panel-max-w-lg">
  <h1>Team</h1>

  <combobox placeholder="Filter by role">
    <option value="eng">Engineering</option>
    <option value="design">Design</option>
  </combobox>

  <card sx="p:4">
    <row gap="3" align="center">
      <avatar></avatar>
      <column gap="1">
        <strong>Ada Lovelace</strong>
        <p sx="fg:muted-foreground text:sm">Engineering</p>
      </column>
      <badge tone="success">Active</badge>
    </row>
  </card>

  <row gap="2">
    <button>Invite</button>
    <button variant="ghost">Export</button>
  </row>
</column>`,
}
