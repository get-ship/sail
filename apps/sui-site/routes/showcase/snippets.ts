// Code snippets for the showcase. Each is the COMPLETE source of the screen
// rendered beside it — no hidden CSS, no imports, no elided parts. Compiled
// by the build (crates/sui/compiler/tests/site_snippets_compile.rs).
export const code = {
  settings: `<column gap="4" sx="p:6">
  <h2>Workspace settings</h2>
  <tabs>
    <tab-list>
      <tab>Profile</tab>
      <tab>Billing</tab>
      <tab>Team</tab>
    </tab-list>
    <tab-panel>
      <column gap="3" sx="pt:4">
        <label>Workspace name</label>
        <input value="Acme Robotics" />
        <label>Plan</label>
        <radio-group>
          <label><input type="radio" name="plan" checked /> Monthly — $12</label>
          <label><input type="radio" name="plan" /> Yearly — $120</label>
        </radio-group>
        <row gap="3" align="center">
          <switch></switch>
          <p sx="text:sm">Email me when a deploy finishes</p>
        </row>
        <separator></separator>
        <row gap="2">
          <button variant="solid" tone="primary">Save changes</button>
          <button variant="ghost">Cancel</button>
        </row>
      </column>
    </tab-panel>
    <tab-panel><p sx="pt:4">Billing settings.</p></tab-panel>
    <tab-panel><p sx="pt:4">Team settings.</p></tab-panel>
  </tabs>
</column>`,

  fleet: `<column gap="4" sx="p:6">
  <row gap="3" align="center">
    <h2 sx="grow:1">Fleet</h2>
    <button variant="outline">Export</button>
    <button variant="solid" tone="primary">Deploy all</button>
  </row>
  <grid cols="3" gap="3">
    <card sx="p:4">
      <badge tone="success">healthy</badge>
      <h3>api-gateway</h3>
      <progress value="82" max="100"></progress>
      <p sx="text:sm fg:muted-foreground">p99 41 ms · 3 replicas</p>
    </card>
    <card sx="p:4">
      <badge tone="warning">degraded</badge>
      <h3>search-index</h3>
      <progress value="47" max="100"></progress>
      <p sx="text:sm fg:muted-foreground">p99 310 ms · rebuilding</p>
    </card>
    <card sx="p:4">
      <badge tone="success">healthy</badge>
      <h3>billing-worker</h3>
      <progress value="94" max="100"></progress>
      <p sx="text:sm fg:muted-foreground">queue empty</p>
    </card>
  </grid>
  <table>
    <thead>
      <tr><th>Deploy</th><th>Service</th><th>Status</th></tr>
    </thead>
    <tbody>
      <tr><td>#214</td><td>api-gateway</td><td><badge tone="success">live</badge></td></tr>
      <tr><td>#213</td><td>search-index</td><td><badge tone="warning">rolling</badge></td></tr>
      <tr><td>#212</td><td>billing-worker</td><td><badge tone="muted">superseded</badge></td></tr>
    </tbody>
  </table>
</column>`,
}
