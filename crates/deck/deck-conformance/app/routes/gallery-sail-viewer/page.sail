<column gap="8">
  style.pad(6).maxW(breakpoint-tablet).mx(auto)

  <header>
    style.borderBWidth(1).borderB(border).padB(4)

    <h1>
      style.fontSize(2xl).fontWeight(bold).fg(foreground).mb(2)
sail/viewer</h1>
    <p>
      style.fg(muted-foreground).fontSize(sm)

      The five viewing primitives: <strong>sail-component-viewer</strong>,
      <strong>sail-prop-knobs</strong>, <strong>sail-prop-matrix</strong>,
      <strong>sail-render</strong>, and <strong>sail-source-panel</strong>.
      Scalar props are demonstrated below. Structured inputs (source signal,
      CEM attrs, compile fn) arrive via <code>*</code> identity at runtime.
    </p>
  </header>

  <!-- ── sail-component-viewer ───────────────────────────────────────── -->
  <column gap="4">
    style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)

    <h2>
      style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)
sail-component-viewer</h2>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      The composition primitive — one tag a host drops in to get a live,
      reactive preview of a Sail component (stage + knobs + matrix + source).
    </p>
    <column gap="4">
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
viewport-width=full (default)</span>
        <sail-component-viewer component-name="sail-button" viewport-width="full"></sail-component-viewer>
      </column>
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
viewport-width=desktop</span>
        <sail-component-viewer component-name="sail-button" viewport-width="desktop"></sail-component-viewer>
      </column>
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
viewport-width=tablet</span>
        <sail-component-viewer component-name="sail-field" viewport-width="tablet"></sail-component-viewer>
      </column>
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
viewport-width=mobile</span>
        <sail-component-viewer component-name="sail-code" viewport-width="mobile"></sail-component-viewer>
      </column>
    </column>
  </column>

  <!-- ── sail-prop-knobs ────────────────────────────────────────────── -->
  <column gap="4">
    style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)

    <h2>
      style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)
sail-prop-knobs</h2>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      The CEM-driven playground — classifies each attribute into an enum
      toggle-group, bool switch, number input, or text input. No scalar
      structural props: knob content arrives via <code>props</code>
      (CEM attrs) and <code>value</code> (current values) at runtime.
    </p>
    <column gap="4">
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
idle (no CEM wired)</span>
        <sail-prop-knobs></sail-prop-knobs>
      </column>
    </column>
  </column>

  <!-- ── sail-prop-matrix ───────────────────────────────────────────── -->
  <column gap="4">
    style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)

    <h2>
      style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)
sail-prop-matrix</h2>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      The Storybook-style state sweep — derives axes from the target CEM and
      builds the full cartesian grid, one <code>sail-render</code> per cell.
    </p>
    <column gap="4">
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
component-name=sail-button</span>
        <sail-prop-matrix component-name="sail-button"></sail-prop-matrix>
      </column>
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
component-name=sail-field</span>
        <sail-prop-matrix component-name="sail-field"></sail-prop-matrix>
      </column>
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
component-name="" (empty state)</span>
        <sail-prop-matrix component-name=""></sail-prop-matrix>
      </column>
    </column>
  </column>

  <!-- ── sail-render ────────────────────────────────────────────────── -->
  <column gap="4">
    style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)

    <h2>
      style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)
sail-render</h2>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      The engine primitive — a live, reactive mount stage. The bound
      <code>compile()</code> and raw <code>source</code> arrive via
      <code>*</code> identity; the three scalar attrs control mode and
      viewport framing.
    </p>
    <column gap="4">
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
mode=source viewport-width=full</span>
        <sail-render mode="source" viewport-width="full"></sail-render>
      </column>
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
mode=tag viewport-width=desktop</span>
        <sail-render mode="tag" component-name="sail-button" viewport-width="desktop"></sail-render>
      </column>
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
mode=source viewport-width=tablet</span>
        <sail-render mode="source" viewport-width="tablet"></sail-render>
      </column>
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
mode=source viewport-width=mobile</span>
        <sail-render mode="source" viewport-width="mobile"></sail-render>
      </column>
    </column>
  </column>

  <!-- ── sail-source-panel ──────────────────────────────────────────── -->
  <column gap="4">
    style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)

    <h2>
      style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)
sail-source-panel</h2>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      Source + paste-able snippet — shows the raw <code>.sail</code> source and
      a recomputed snippet that default-diffs the current prop values. The Copy
      button writes the snippet to the clipboard.
    </p>
    <column gap="4">
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
component-name=sail-button lang=html</span>
        <sail-source-panel component-name="sail-button" lang="html"></sail-source-panel>
      </column>
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
component-name=sail-field lang=html</span>
        <sail-source-panel component-name="sail-field" lang="html"></sail-source-panel>
      </column>
      <column gap="2">
        style.borderWidth(1).border(divider).radius(md).pad(3).bg(surface-2)

        <span>
          style.fontSize(xs).fontWeight(medium).fg(muted-foreground).textCase(lowercase).tracking(wide).fontFamily(mono)
component-name=sail-code lang=html</span>
        <sail-source-panel component-name="sail-code" lang="html"></sail-source-panel>
      </column>
    </column>
  </column>
</column>
