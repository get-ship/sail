<column gap="8">
  style.pad(6).maxW(breakpoint-tablet).mx(auto)

  <header>
    style.borderBWidth(1).borderB(border).padB(4)

    <h1>
      style.fontSize(2xl).fontWeight(bold).fg(foreground).mb(2)
ship/pdf — Domain Stubs</h1>
    <p>
      style.fg(muted-foreground).fontSize(sm)

      Pack <strong>ship/pdf</strong>. Each card below renders the real custom
      element tag with static mock attribute values so the shell registers and
      paints. A visible badge on each card marks it as a domain stub — no
      <code>ship.*</code> calls, no live data.
    </p>
  </header>

  <!-- ship-pdf -->
  <column gap="4">
    style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)

    <h2>
      style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)

      <code>ship-pdf</code>
      <span>
        style.fontSize(xs).fontWeight(medium).fontFamily(body).fg(warning-foreground).bg(warning).radius(sm).padX(2).padY(1).tracking(wide).ms(3)
domain-stub — awaits backend spec</span>
    </h2>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      Props: <code>src: string</code> (URL or blob reference, default empty),
      <code>page: number</code> (1-based page index, default 1),
      <code>zoom: number</code> (scale factor, default 1),
      <code>label: string</code> (accessible label, default "PDF page").
      Slot: <code>toolbar</code> (custom toolbar controls). No live data is
      wired in this stub — <code>src</code> is intentionally empty so the
      component renders its idle/empty shell state.
    </p>

    <column gap="4">
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
default — no src, page=1, zoom=1</span>
        <ship-pdf label="PDF page"></ship-pdf>
      </column>

      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
page=2 zoom=1.5 — page and zoom attrs reflected</span>
        <ship-pdf page="2" zoom="1.5" label="PDF page 2 at 150%"></ship-pdf>
      </column>

      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
page=1 zoom=0.75 — reduced zoom</span>
        <ship-pdf page="1" zoom="0.75" label="PDF page at 75%"></ship-pdf>
      </column>

      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
with toolbar slot — custom controls slotted</span>
        <ship-pdf page="1" zoom="1" label="PDF with toolbar">
          <row slot="toolbar" gap="3">
            style.pad(2).bg(surface-1).radius(sm).borderWidth(1).border(border)

            <button disabled>
              style.fontSize(sm).padX(2).padY(1).borderWidth(1).border(border).radius(sm).bg(surface-2).fg(muted-foreground).cursor(default)
&#8592; Prev</button>
            <span>
              style.fontSize(sm).fg(foreground).fontFamily(mono)
Page 1</span>
            <button disabled>
              style.fontSize(sm).padX(2).padY(1).borderWidth(1).border(border).radius(sm).bg(surface-2).fg(muted-foreground).cursor(default)
Next &#8594;</button>
            <span>
              style.fontSize(xs).fg(muted-foreground).italic(true).ml(auto)
slotted toolbar demo</span>
          </row>
        </ship-pdf>
      </column>
    </column>
  </column>
</column>
