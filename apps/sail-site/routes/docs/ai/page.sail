<!--
  /docs/ai — agent-first product section landing (static).
-->

<scope>
  style page = { box: space({ pad: 0, mx: auto }).size({ max-w: breakpoint-tablet }) }
  style eyebrow = { box: space({ mb: 2 }), paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style callout = { box: space({ pad: 3, pad-x: 4 }), paint: border({ color: border, width: 1 }).border({ color: primary, width: 3, edge: l }).radius(md).ink({ color: muted-foreground }), text: type({ size: sm }) }
  style footNav = { box: space({ pad-t: 4, mt: 8 }), paint: border({ color: border, width: 1, edge: t }) }
</scope>

<column style(page)>
  <p style(eyebrow)>AI</p>
  <h1>Sail is built for agents</h1>
  <p style(lead)>
    Most UI frameworks optimize for a human at a keyboard. Sail optimizes for a
    model: a narrow grammar, a strict lint that turns design-system violations
    into build errors, a tight compile-check loop, and a batteries-included
    standard library — so an agent produces correct, on-brand UI on the first pass.
  </p>

  <h2>Why agent-first changes the design</h2>
  <p>
    An agent doesn’t need ergonomic sugar; it needs fast, unambiguous feedback
    and guardrails it can’t drift past. Sail leans into that:
  </p>
  <ul>
    <li><strong>Strict lint as build errors</strong> — off-token color, ad-hoc spacing, a raw <code>&lt;div&gt;</code> where a real primitive exists: rejected at compile time, not review time.</li>
    <li><strong>Tight compile loop</strong> — one command, structured errors with source spans; the model reads the error and fixes it.</li>
    <li><strong>Batteries included</strong> — ~74 accessible primitives plus a design system, so the agent composes instead of reinventing.</li>
    <li><strong>Narrow grammar</strong> — a small, regular surface over HTML/TS/CSS the model already knows means fewer ways to be wrong.</li>
  </ul>

  <h2>The AI toolchain</h2>
  <p>The tools that make an agent fluent in Sail — usable together or on their own:</p>
  <ul>
    <li><strong>MCP server</strong> — serves the live docs and runs static analysis to catch common generative pitfalls before they ship.</li>
    <li><strong>Skills</strong> — lazy-loaded briefs that teach an agent Sail’s idioms and when to reach for which primitive.</li>
    <li><strong>LSP</strong> — the compiler’s diagnostics in-editor, for the human half of the loop.</li>
    <li><strong>Eval harness</strong> — a scored Sail-authoring benchmark, so changes to the language or lint are measured against whether agents actually get it right.</li>
  </ul>

  <p style(callout)>
    <strong>This is the headline, not an appendix.</strong> "Most ergonomic for
    agents" is the thesis of the whole framework; this section is where it gets
    concrete. Instructions, MCP setup, Skills, and the eval harness land here.
  </p>

  <row style(footNav)>
    <a variant="default" href="/">← SHIP UI</a>
  </row>
</column>
