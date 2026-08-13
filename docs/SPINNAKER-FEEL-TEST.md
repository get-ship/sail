# SPINNAKER FEEL TEST — FINAL FORM (channels dissolved, ratified 08-13)

The ratification read, final syntax. One language at every size:
a style is a FLAT CHAIN of calls; `when:`/`match:`/`animate:` are the
only keyed members. Rounds 1–6 are superseded.

---

## 1. Form — profile settings

```sail
<scope>
  style formCard = { size(full).pad({ x: 5, y: 4 }).bg(card).border(subtle).radius(lg) }
  style muted = { type(body-sm).fg(muted-foreground) }
</scope>

<card style(formCard)>
  <column gap="4">
    <row justify="between" align="center">
      <h2 style(type(heading-2))>Profile</h2>
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
```

## 2. Overlay — dropdown panel + confirm dialog

```sail
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
    <h3 style(type(heading-3))>Delete project?</h3>
    <p style(muted)>This permanently removes the project and its deployments.</p>
    <row gap="2" justify="end">
      <button variant="outline" on:click={dismiss}>Cancel</button>
      <button tone="destructive" on:click={destroy}>Delete</button>
    </row>
  </column>
</dialog>
```

## 3. Nav — sticky docs topbar

```sail
<scope>
  style topbar = {
    position({ scheme: sticky, t: 0, z: header }).size(full).pad({ x: 4 })
      .bg(background).border({ color: border-subtle, edge: bottom }),
    animate: { transition: play(colors, fast) },
  }
  style navLink = {
    pad({ x: 3, y: 2 }).radius(md).type(label),
    when: {
      hover: bg(surface-2),
      current: bg(primary).fg(primary-foreground),
    },
  }
</scope>

<navbar style(topbar)>
  <row gap="1" align="center">
    <a href="/" style(type(heading-3))>sail</a>
    <a href="/docs" style(navLink)>Docs</a>
    <a href="/components" style(navLink)>Components</a>
    <a href="/playground" style(navLink)>Playground</a>
    <row gap="2" style(margin({ l: auto }))>
      <theme-picker />
      <button size="sm">Get started</button>
    </row>
  </row>
</navbar>
```

## 4. Data display — stat cards + table

```sail
<scope>
  style statValue = { type(display-sm) }
  style statLabel = { type(label).fg(muted-foreground) }
  style tableWrap = { size(full).overflow({ x: auto }).border(subtle).radius(lg) }
  style rowHover = { when: { hover: bg(surface-1) } }
</scope>

<grid cols="3" gap="4">
  <card style(pad({ x: 4, y: 3 }))>
    <p style(statLabel)>Deployments</p>
    <p style(statValue)>1,284</p>
    <p style(type(body-sm).fg(success))>+12% this week</p>
  </card>
</grid>

<card>
  <table style(tableWrap)>
    <thead><tr><th>Service</th><th>Status</th><th>Latency</th></tr></thead>
    <tbody>
      <for {services()} as="svc" key={svc.id}>
        <tr style(rowHover)>
          <td>{svc.name}</td>
          <td><badge tone={svc.tone}>{svc.status}</badge></td>
          <td style(type(mono-sm))>{svc.latency}ms</td>
        </tr>
      </for>
    </tbody>
  </table>
</card>
```

## 5. Motion-heavy — toast + hover-lift card

```sail
<scope>
  style toastCard = {
    size({ min-w: 80 }).pad({ x: 4, y: 3 }).bg(card).border(subtle).radius(lg).elevation(lg),
    animate: {
      enter: play(slide-up, fast),
      exit: play(fade, fast),
      transition: play(colors, fast),
    },
  }
  style liftCard = {
    bg(card).border(subtle).radius(lg).elevation(sm),
    animate: { transition: play({ props: all, duration: fast, ease: out }) },
    when: {
      hover: elevation(xl).translateY(-1),
      press: scale(0_98),
    },
  }
</scope>

<toast-region placement="bottom-right">
  <for {toasts()} as="t" key={t.id}>
    <toast tone={t.tone} style(toastCard)>{t.message}</toast>
  </for>
</toast-region>

<card style(liftCard)>
  <img src={item.thumb} alt={item.title}
       style(size(full).radius({ t: lg })) />
  <column gap="1" style(pad({ x: 4, y: 3 }))>
    <h3 style(type(heading-3))>{item.title}</h3>
    <p style(muted)>{item.blurb}</p>
  </column>
</card>
```

---

## The anchor (densest real-corpus case, full evolution)

```sail
// v1 (today's corpus):
style departedBadge = { box: padX(1), paint: fg(muted-foreground).radius(sm),
  text: fontSize(xs).fontFamily(mono),
  when: { stateLive: { paint: bg(primary).fg(primary-foreground) } } }

// final:
style liveBadge = {
  pad({ x: 1 }).fg(muted-foreground).radius(sm).type({ family: mono, size: xs }),
  when: { stateLive: bg(primary).fg(primary-foreground) },
}
// theme-owned:  type(mono-badge)  — one reference.
```

## The form ladder (one language at every size)

```sail
style(type(heading-2))                     // inline, one call
style(margin({ l: auto }))                 // inline, slot object
style(navLink)                             // named application
style(card-chrome, elevation(xl))          // composition with overrides
style navLink = { …, when: { … } }         // declared, states
// theme named-style (studio-edited):
style(card-chrome)                         // reference only
```
