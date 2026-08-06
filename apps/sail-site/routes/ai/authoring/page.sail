<!--
  /ai/authoring — the agent authoring loop, adapted from
  packages/sail/tutorial/src/prose/09-agent-loop/index.md. The source is a
  tutorial lesson (a step in the 25-lesson track); here it lands as a
  top-level AI section page grounded in that prose.
-->

<controller>
// /ai/authoring — the agent authoring loop, adapted from the
// 09-agent-loop tutorial lesson. Snippets live in ./snippets so the literal
// loop-diagram ASCII never leaks into the inferred scope.
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { loopDiagram } from './snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ loopDiagram }))
</controller>

<article>
  style({ box: maxW(breakpoint-tablet).mx(auto) })

  <p>
    style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })
AI</p>
  <h1>The agent authoring loop</h1>
  <p>
    style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })

    An agent authoring a Sail route produces a <code>.sail</code> +
    <code>controller.ts</code> pair that compiles, stays in scope, and behaves
    to spec — all in one turn. Ship gives the agent six surfaces to make that
    one-turn target reachable.
  </p>

  <pre>
    style({ box: pad(4).padX(5), paint: bg(surface-1).borderWidth(1).border(border).radius(lg), text: fontFamily(mono).fontSize(sm).leading(relaxed).whitespace(pre-wrap).break(words) })
{loopDiagram}</pre>

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
    See <code>docs/ship-sail/references/docs/agent-surfaces.md</code> for the
    full tool inventory. See
    <code>docs/ship-sail/references/docs/agent-authoring.md</code> for the
    do-now checklist (filename rules, the <code>&lt;data&gt;</code> directive
    shape, the right places to put <code>&lt;style&gt;</code>).
  </p>

  <row>
    style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })

    <a href="/ai">← Overview</a>
    <a href="/ai/tooling">Tooling →</a>
  </row>
</article>
