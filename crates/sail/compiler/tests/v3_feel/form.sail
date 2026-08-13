<scope>
  style formCard = { size(full).pad({ x: 5, y: 4 }).bg(card).border(subtle).radius(lg) }
  style muted = { type(body-sm).fg(muted-foreground) }
</scope>

<card style(formCard)>
  <column gap="4">
    <row justify="between" align="center">
      <h2>Profile</h2>
      <badge tone="muted">Free plan</badge>
    </row>
    <field label="Display name" hint="Shown on your public page">
      <input type="text" bind:value={name()} placeholder="Ada Lovelace" />
    </field>
    <field label="Bio"><textarea rows="4" bind:value={bio()} /></field>
    <row gap="2" justify="end" style(pad({ t: 2 }))>
      <button variant="outline" on:click={reset}>Reset</button>
      <button on:click={save} disabled={saving()}>{saving() ? 'Saving…' : 'Save changes'}</button>
    </row>
  </column>
</card>
