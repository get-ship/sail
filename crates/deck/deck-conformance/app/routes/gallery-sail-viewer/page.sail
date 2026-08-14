<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style pageHead = { pad({ b: 4 }).border({ width: 1, color: border, edge: b }) }
  style title = { margin({ b: 2 }).fg(foreground).type({ size: 2xl, weight: bold }) }
  style lede = { fg(muted-foreground).type({ size: sm }) }
  style card = { pad(5).border({ width: 1, color: border }).radius(lg).bg(surface-1) }
  style cardTitle = { fg(foreground).type({ size: lg, weight: semibold, family: mono }) }
  style body = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style stage = { pad(3).border({ width: 1, color: divider }).radius(md).bg(surface-2) }
  style caption = { fg(muted-foreground).type({ size: xs, weight: medium, case: lowercase, tracking: wide, family: mono }) }
</scope>

<column gap="8" style(page)>
  <header style(pageHead)>
    <h1 style(title)>sail/viewer</h1>
    <p style(lede)>
      The five viewing primitives: <strong>sail-component-viewer</strong>,
      <strong>sail-prop-knobs</strong>, <strong>sail-prop-matrix</strong>,
      <strong>sail-render</strong>, and <strong>sail-source-panel</strong>.
      Scalar props are demonstrated below. Structured inputs (source signal,
      CEM attrs, compile fn) arrive via <code>*</code> identity at runtime.
    </p>
  </header>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>sail-component-viewer</h2>
    <p style(body)>
      The composition primitive — one tag a host drops in to get a live,
      reactive preview of a Sail component (stage + knobs + matrix + source).
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>viewport-width=full (default)</span>
        <sail-component-viewer component-name="sail-button" viewport-width="full"></sail-component-viewer>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>viewport-width=desktop</span>
        <sail-component-viewer component-name="sail-button" viewport-width="desktop"></sail-component-viewer>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>viewport-width=tablet</span>
        <sail-component-viewer component-name="sail-field" viewport-width="tablet"></sail-component-viewer>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>viewport-width=mobile</span>
        <sail-component-viewer component-name="sail-code" viewport-width="mobile"></sail-component-viewer>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>sail-prop-knobs</h2>
    <p style(body)>
      The CEM-driven playground — classifies each attribute into an enum
      toggle-group, bool switch, number input, or text input. No scalar
      structural props: knob content arrives via <code>props</code>
      (CEM attrs) and <code>value</code> (current values) at runtime.
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>idle (no CEM wired)</span>
        <sail-prop-knobs></sail-prop-knobs>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>sail-prop-matrix</h2>
    <p style(body)>
      The Storybook-style state sweep — derives axes from the target CEM and
      builds the full cartesian grid, one <code>sail-render</code> per cell.
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>component-name=sail-button</span>
        <sail-prop-matrix component-name="sail-button"></sail-prop-matrix>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>component-name=sail-field</span>
        <sail-prop-matrix component-name="sail-field"></sail-prop-matrix>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>component-name="" (empty state)</span>
        <sail-prop-matrix component-name=""></sail-prop-matrix>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>sail-render</h2>
    <p style(body)>
      The engine primitive — a live, reactive mount stage. The bound
      <code>compile()</code> and raw <code>source</code> arrive via
      <code>*</code> identity; the three scalar attrs control mode and
      viewport framing.
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>mode=source viewport-width=full</span>
        <sail-render mode="source" viewport-width="full"></sail-render>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>mode=tag viewport-width=desktop</span>
        <sail-render mode="tag" component-name="sail-button" viewport-width="desktop"></sail-render>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>mode=source viewport-width=tablet</span>
        <sail-render mode="source" viewport-width="tablet"></sail-render>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>mode=source viewport-width=mobile</span>
        <sail-render mode="source" viewport-width="mobile"></sail-render>
      </column>
    </column>
  </column>
  <column gap="4" style(card)>
    <h2 style(cardTitle)>sail-source-panel</h2>
    <p style(body)>
      Source + paste-able snippet — shows the raw <code>.sail</code> source and
      a recomputed snippet that default-diffs the current prop values. The Copy
      button writes the snippet to the clipboard.
    </p>
    <column gap="4">
      <column gap="2" style(stage)>
        <span style(caption)>component-name=sail-button lang=html</span>
        <sail-source-panel component-name="sail-button" lang="html"></sail-source-panel>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>component-name=sail-field lang=html</span>
        <sail-source-panel component-name="sail-field" lang="html"></sail-source-panel>
      </column>
      <column gap="2" style(stage)>
        <span style(caption)>component-name=sail-code lang=html</span>
        <sail-source-panel component-name="sail-code" lang="html"></sail-source-panel>
      </column>
    </column>
  </column>
</column>
