<scope>
  style menuPanel = {
    position({ scheme: absolute, t: full, r: 0 }).size({ min-w: 48 })
      .bg(popover).border(subtle).radius(md).elevation(lg),
    animate: { enter: play(fade-scale-in, fast), exit: play(fade, fast) },
  }
</scope>

<dropdown>
  <button variant="outline">Workspace <icon name="chevron-down" /></button>
  <menu-item style(menuPanel)>
    <a href="/settings">Settings</a>
    <separator />
    <a href="/logout" style(fg(destructive))>Log out</a>
  </menu-item>
</dropdown>

<dialog open={confirm()} on:close={dismiss}>
  <column gap="3">
    <h3>Delete project?</h3>
    <p style(type(body-sm).fg(muted-foreground))>This permanently removes the project and its deployments.</p>
    <row gap="2" justify="end">
      <button variant="outline" on:click={dismiss}>Cancel</button>
      <button tone="destructive" on:click={destroy}>Delete</button>
    </row>
  </column>
</dialog>
