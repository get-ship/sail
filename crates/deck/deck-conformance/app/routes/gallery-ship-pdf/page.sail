<!-- ship/pdf domain stubs — static mock attrs; empty src paints idle shell. -->

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style header = { pad({ b: 4 }).border({ width: 1, color: border, edge: b }) }
  style title = { margin({ b: 2 }).fg(foreground).type({ size: 2xl, weight: bold }) }
  style lead = { fg(muted-foreground).type({ size: sm }) }
  style card = { pad(5).border({ width: 1, color: border }).radius(lg).bg(surface-1) }
  style cardTitle = { fg(foreground).type({ size: lg, weight: semibold, family: mono }) }
  style body = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style arm = { pad(3).border({ width: 1, color: divider }).radius(md).bg(surface-2) }
  style caption = { fg(muted-foreground).type({ size: xs, weight: medium, case: lowercase, tracking: wide, family: mono }) }
  style toolbar = { pad(2).bg(surface-1).radius(sm).border({ width: 1, color: border }) }
  style toolBtn = { pad({ x: 2, y: 1 }).border({ width: 1, color: border }).radius(sm).bg(surface-2).fg(muted-foreground).cursor(default).type({ size: sm }) }
  style pageLabel = { fg(foreground).type({ size: sm, family: mono }) }
  style toolHint = { margin({ l: auto }).fg(muted-foreground).type({ size: xs, italic: true }) }
</scope>

<column gap="8" style(page)>
  <header style(header)>
    <h1 style(title)>ship/pdf — Domain Stubs</h1>
    <p style(lead)>
      Pack <strong>ship/pdf</strong>. Each card below renders the real custom
      element tag with static mock attribute values so the shell registers and
      paints. A visible badge on each card marks it as a domain stub — no
      <code>ship.*</code> calls, no live data.
    </p>
  </header>

  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-pdf</code>
      <badge tone="warning" size="sm">domain-stub</badge>
    </h2>
    <p style(body)>
      Props: <code>src: string</code> (URL or blob reference, default empty),
      <code>page: number</code> (1-based page index, default 1),
      <code>zoom: number</code> (scale factor, default 1),
      <code>label: string</code> (accessible label, default "PDF page").
      Slot: <code>toolbar</code> (custom toolbar controls). No live data is
      wired in this stub — <code>src</code> is intentionally empty so the
      component renders its idle/empty shell state.
    </p>

    <column gap="4">
      <column gap="2" style(arm)>
        <span style(caption)>default — no src, page=1, zoom=1</span>
        <ship-pdf label="PDF page"></ship-pdf>
      </column>

      <column gap="2" style(arm)>
        <span style(caption)>page=2 zoom=1.5 — page and zoom attrs reflected</span>
        <ship-pdf page="2" zoom="1.5" label="PDF page 2 at 150%"></ship-pdf>
      </column>

      <column gap="2" style(arm)>
        <span style(caption)>page=1 zoom=0.75 — reduced zoom</span>
        <ship-pdf page="1" zoom="0.75" label="PDF page at 75%"></ship-pdf>
      </column>

      <column gap="2" style(arm)>
        <span style(caption)>with toolbar slot — custom controls slotted</span>
        <ship-pdf page="1" zoom="1" label="PDF with toolbar">
          <row slot="toolbar" gap="3" style(toolbar)>
            <button disabled style(toolBtn)>&#8592; Prev</button>
            <span style(pageLabel)>Page 1</span>
            <button disabled style(toolBtn)>Next &#8594;</button>
            <span style(toolHint)>slotted toolbar demo</span>
          </row>
        </ship-pdf>
      </column>
    </column>
  </column>
</column>
