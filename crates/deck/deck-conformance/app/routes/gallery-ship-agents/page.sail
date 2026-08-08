<!-- ship/agents domain stubs — static tags for shell registration, no live data. -->

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style header = { box: padB(4), paint: borderBWidth(1).borderB(border) }
  style title = { box: mb(2), paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style lead = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style card = { box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) }
  style cardTitle = { paint: fg(foreground), text: fontSize(lg).fontWeight(semibold).fontFamily(mono) }
  style body = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style rowLabel = { box: minW(20), paint: fg(muted-foreground), text: fontFamily(mono).fontSize(xs) }
  style slotBtn = { box: padY(1).padX(3), paint: borderWidth(1).border(border).radius(sm).bg(surface-2).fg(foreground).cursor(default), text: fontSize(sm) }
  type HarnessRowProps = {
    label: string
    size: string
    variant: string
  }
</scope>

<partial harness-row props: HarnessRowProps>
  <row gap="3">
    <span style(rowLabel)>{label}</span>
    <sail-harness-blob harness="claude" size={size} variant={variant}>Claude</sail-harness-blob>
    <sail-harness-blob harness="codex" size={size} variant={variant}>Codex</sail-harness-blob>
    <sail-harness-blob harness="cursor" size={size} variant={variant}>Cursor</sail-harness-blob>
    <sail-harness-blob harness="gemini" size={size} variant={variant}>Gemini</sail-harness-blob>
    <sail-harness-blob harness="opencode" size={size} variant={variant}>OpenCode</sail-harness-blob>
  </row>
</partial>

<column gap="8" style(page)>
  <header style(header)>
    <h1 style(title)>ship/agents — Domain Stubs</h1>
    <p style(lead)>
      Pack <strong>ship/agents</strong>. Each card below renders the real custom
      element tag with static mock attribute values so the shell registers and
      paints. A visible badge on each card marks it as a domain stub — no
      <code>ship.*</code> calls, no live data.
    </p>
  </header>

  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-agent-card</code>
      <badge tone="warning" size="sm" pill>domain-stub</badge>
    </h2>
    <p style(body)>
      Props: <code>id: string</code>. Slot: <code>trailing</code> (action buttons).
      Production uses <code>ship.agents.detail</code> via <code>live()</code>; this stub
      passes a static mock id.
    </p>
    <column gap="4">
      <ship-agent-card id="agent-mock-001">
        <button slot="trailing" style(slotBtn)>Edit</button>
      </ship-agent-card>
    </column>
  </column>

  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-agent-thread</code>
      <badge tone="warning" size="sm" pill>domain-stub</badge>
    </h2>
    <p style(body)>
      Props: <code>session-id: string</code>. No slots.
      Composes <code>sail-message-list</code> + <code>sail-composer</code>;
      the Ship wrapper owns the session binding and submit handler.
    </p>
    <column gap="4">
      <ship-agent-thread session-id="sess-mock-001"></ship-agent-thread>
    </column>
  </column>

  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-agents-table</code>
      <badge tone="warning" size="sm" pill>domain-stub</badge>
    </h2>
    <p style(body)>
      No props — data is fetched internally via <code>live()</code> from
      <code>ship.agents.summaries()</code>. Renders a sortable
      <code>sail-table</code> with columns: name, description, source, skills, mcp.
    </p>
    <column gap="4">
      <ship-agents-table></ship-agents-table>
    </column>
  </column>

  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>sail-harness-blob</code>
      <badge tone="warning" size="sm" pill>domain-stub</badge>
    </h2>
    <p style(body)>
      Props: <code>harness: claude|codex|cursor|gemini|opencode</code>,
      <code>size: sm|md|lg</code>, <code>variant: tile|glyph</code>.
      Slot: default (label text). Inlines brand SVG — no network fetch.
    </p>

    <column gap="4">
      <harness-row label="tile / md" size="md" variant="tile" />
      <harness-row label="tile / sm" size="sm" variant="tile" />
      <harness-row label="tile / lg" size="lg" variant="tile" />
      <harness-row label="glyph / md" size="md" variant="glyph" />
    </column>
  </column>
</column>
