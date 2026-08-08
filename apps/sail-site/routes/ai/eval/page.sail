<!--
  /ai/eval — scoring with sail-eval, adapted from
  packages/sail/tutorial/src/prose/11-sail-eval/index.md. The source covers a
  sub-scores table and a sniffEscapeHatches() helper; both land here. The
  sub-score table is rendered as sub-headings under "Sub-scores" so the page
  stays prose-only and requires no co-located stylesheet.
-->

<controller>
// /ai/eval — scoring with sail-eval, adapted from the 11-sail-eval tutorial
// lesson. Snippets live in ./snippets so literal braces in the example block
// and the sniffEscapeHatches output never leak into the inferred scope.
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { evalExample, hatchExample } from '../../../src/compositions/ai-eval-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ evalExample, hatchExample }))
</controller>

<article style({ box: maxW(breakpoint-tablet).mx(auto) })>
  <p style({ box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) })>AI</p>
  <h1>Scoring with sail-eval</h1>
  <p style({ paint: fg(muted-foreground), text: fontSize(lg).leading(relaxed) })>
    <code>@ship/sail-eval</code> runs an agent’s output against a behavioural
    spec and returns a structured quality score. The sub-scores tell you
    which gate (compile / scope / behaviour) the agent missed, so the next
    turn’s prompt can target the failure.
  </p>

  <h2>The API, in one block</h2>
  <code-block code={evalExample} lang="ts"></code-block>

  <h2>Sub-scores</h2>
  <h3><code>compile</code></h3>
  <p>
    Does the generated <code>.sail</code> + controller compile via the wasm
    pipeline? Failure looks like: compile errors from <code>sail-compiler</code>
    or controller parse errors.
  </p>
  <h3><code>scope</code></h3>
  <p>
    Does every signal, directive, and tag the template references actually
    resolve? Failure looks like: <code>unknown identifier</code>,
    <code>&lt;sail-foo&gt;</code> not in catalog, <code>bind:value</code> against
    a non-signal.
  </p>
  <h3><code>behaviour</code></h3>
  <p>
    Does the mounted route in jsdom pass every interaction + assert in the
    spec? Failure looks like: click → no DOM change, assertion text mismatch,
    signal never updated.
  </p>
  <p>
    The overall <code>score</code> is a weighted average (compile is
    hardest-gated: <code>compile=0</code> caps the whole score regardless of
    behaviour).
  </p>

  <h2>sniffEscapeHatches</h2>
  <p>
    <code>sniffEscapeHatches(output)</code> returns a separate array of policy
    violations — things that compile but shouldn’t ship:
  </p>
  <code-block code={hatchExample} lang="ts"></code-block>
  <p>Detected hatches today:</p>
  <ul>
    <li>raw <code>&lt;script&gt;</code> tags in templates</li>
    <li><code>@ts-ignore</code> / <code>@ts-expect-error</code> in controllers</li>
    <li><code>eval()</code> calls anywhere</li>
    <li>imports outside the harness’s importmap (e.g. node-only modules)</li>
  </ul>

  <h2>Why this matters</h2>
  <p>
    <code>sail-eval</code> is the <strong>assertion</strong> an agent makes
    about its own output before reporting done. Diagnostics tell you "this
    compiles"; eval tells you "this <em>works</em>". The combination is what
    makes one-turn authoring viable.
  </p>
  <p>
    See <code>docs/ship-sail/references/docs/agent-surfaces.md</code> § 3 for
    the full API surface.
  </p>

  <row style({ box: mt(8).padT(4), paint: borderTWidth(1).borderT(border) })>
    <a href="/ai/tooling">← Tooling</a>
  </row>
</article>
