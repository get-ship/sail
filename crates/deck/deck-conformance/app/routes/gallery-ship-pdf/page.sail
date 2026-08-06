<column gap="8">
  style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })

  <header>
    style({ box: padB(4), paint: borderBWidth(1).borderB(border) })

    <h1>
      style({ box: mb(2), paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })
ship/pdf — Domain Stubs</h1>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm) })

      Pack <strong>ship/pdf</strong>. Each card below renders the real custom
      element tag with static mock attribute values so the shell registers and
      paints. A visible badge on each card marks it as a domain stub — no
      <code>ship.*</code> calls, no live data.
    </p>
  </header>

  <!-- ship-pdf -->
  <column gap="4">
    style({ box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })

    <h2>
      style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold).fontFamily(mono) })

      <code>ship-pdf</code>
      <span>
        style({ box: padX(2).padY(1).ms(3), paint: fg(warning-foreground).bg(warning).radius(sm), text: fontSize(xs).fontWeight(medium).fontFamily(body).tracking(wide) })
domain-stub — awaits backend spec</span>
    </h2>
    <p>
      style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) })

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
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
default — no src, page=1, zoom=1</span>
        <ship-pdf label="PDF page"></ship-pdf>
      </column>

      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
page=2 zoom=1.5 — page and zoom attrs reflected</span>
        <ship-pdf page="2" zoom="1.5" label="PDF page 2 at 150%"></ship-pdf>
      </column>

      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
page=1 zoom=0.75 — reduced zoom</span>
        <ship-pdf page="1" zoom="0.75" label="PDF page at 75%"></ship-pdf>
      </column>

      <column gap="2">
        style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })

        <span>
          style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })
with toolbar slot — custom controls slotted</span>
        <ship-pdf page="1" zoom="1" label="PDF with toolbar">
          <row slot="toolbar" gap="3">
            style({ box: pad(2), paint: bg(surface-1).radius(sm).borderWidth(1).border(border) })

            <button disabled>
              style({ box: padX(2).padY(1), paint: borderWidth(1).border(border).radius(sm).bg(surface-2).fg(muted-foreground).cursor(default), text: fontSize(sm) })
&#8592; Prev</button>
            <span>
              style({ paint: fg(foreground), text: fontSize(sm).fontFamily(mono) })
Page 1</span>
            <button disabled>
              style({ box: padX(2).padY(1), paint: borderWidth(1).border(border).radius(sm).bg(surface-2).fg(muted-foreground).cursor(default), text: fontSize(sm) })
Next &#8594;</button>
            <span>
              style({ box: ml(auto), paint: fg(muted-foreground), text: fontSize(xs).italic(true) })
slotted toolbar demo</span>
          </row>
        </ship-pdf>
      </column>
    </column>
  </column>
</column>
