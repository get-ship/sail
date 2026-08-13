<!--
  /docs/ai/authoring — the agent authoring loop (diagnose → fix → compile → score).
-->

<scope>
  style page = { box: size({ max-w: breakpoint-tablet }).space({ mx: auto }) }
  style eyebrow = { box: space({ mb: 2 }), paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style diagram = { box: space({ pad: 4, pad-x: 5 }), paint: fill({ color: surface-1 }).border({ color: border, width: 1 }).radius(lg), text: type({ family: mono, size: sm, leading: relaxed }).whitespace(pre-wrap).break(words) }
  style footNav = { box: space({ pad-t: 4, mt: 8 }), paint: border({ color: border, width: 1, edge: t }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { loopDiagram } from '../../../../src/compositions/ai-authoring-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ loopDiagram }))
</controller>

<article style(page)>
  <p style(eyebrow)>AI</p>
  <h1>The agent authoring loop</h1>
  <p style(lead)>
    An agent authoring a Sail route produces a <code>.sail</code> file with an
    inline <code>&lt;controller&gt;</code> that compiles, stays in scope, and
    behaves to spec — all in one turn. Ship gives the agent six surfaces to make
    that one-turn target reachable.
  </p>

  <pre style(diagram)>{loopDiagram}</pre>

  <p>
    The example to the right is a route with <strong>four deliberate
    diagnostics</strong> baked in: an unknown event modifier, a missing prop
    declaration, a stale signal reference, and a render-subset escape
    (<code>.toFixed()</code> in the template). An agent would:
  </p>
  <ol>
    <li>
      <strong>Diagnose</strong> —
      <code>cargo run -p sail-lint --example check -- page.sail</code> returns
      findings, each with <code>line</code>, <code>col</code>,
      <code>severity</code>, <code>message</code>.
    </li>
    <li>
      <strong>Fix</strong> — one round-trip per error category. The agent edits
      the file (the editor pane) and re-diagnoses.
    </li>
    <li>
      <strong>Compile-check</strong> — <code>sail-compiler build page.sail</code>
      runs the full lowering pipeline and surfaces anything the lint pass
      missed (lowering-only failures). A non-zero exit is a hard failure.
    </li>
    <li>
      <strong>Score</strong> — once the route compiles, <code>@ship/sail-eval</code>
      runs it in jsdom against a behavioural spec and returns
      <code>&#123; score, sub, report &#125;</code>. The agent decides whether
      to retry based on the score.
    </li>
  </ol>

  <h2>Why this matters</h2>
  <p>
    The point of the loop is <strong>single-turn confidence</strong>: the agent
    doesn’t guess and submit. It writes, diagnoses, fixes the surfaced issues,
    re-diagnoses until clean, then scores. If the score is too low, it
    rewrites — but the compile path is never the blocker.
  </p>
  <p>
    See the AI <a href="/docs/ai/tooling">Tooling</a> and
    <a href="/docs/ai/eval">Eval</a> pages for the lint/compile and scoring surfaces.
  </p>

  <row style(footNav)>
    <a href="/docs/ai">← Overview</a>
    <a href="/docs/ai/tooling">Tooling →</a>
  </row>
</article>
