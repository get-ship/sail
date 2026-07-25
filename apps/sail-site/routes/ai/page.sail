<controller>
// /ai — the AI product section of the Ship UI site. The headline section for an
// agent-first framework. Theme owned by the shell.
import { controller } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

export default controller(() => ({}))
</controller>

<column sx={style.pad(0).maxW(breakpoint-tablet).mx(auto)}>
  <p sx={style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)}>AI</p>
  <h1>Sail is built for agents</h1>
  <p sx={style.fontSize(lg).fg(muted-foreground).leading(relaxed)}>
    Most UI frameworks optimize for a human at a keyboard. Sail optimizes for a
    model: a narrow grammar, a strict lint that turns design-system violations
    into build errors, a tight compile-check loop, and a batteries-included
    standard library — so an agent produces correct, on-brand UI on the first pass.
  </p>

  <h2>Why agent-first changes the design</h2>
  <p>
    An agent doesn't need ergonomic sugar; it needs fast, unambiguous feedback
    and guardrails it can't drift past. Sail leans into that:
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
    <li><strong>Skills</strong> — lazy-loaded briefs that teach an agent Sail's idioms and when to reach for which primitive.</li>
    <li><strong>LSP</strong> — the compiler's diagnostics in-editor, for the human half of the loop.</li>
    <li><strong>Eval harness</strong> — a scored Sail-authoring benchmark, so changes to the language or lint are measured against whether agents actually get it right.</li>
  </ul>

  <p sx={style.borderWidth(1).border(border).borderLWidth(3).borderL(primary).radius(md).pad(3).padX(4).fontSize(sm).fg(muted-foreground)}>
    <strong>This is the headline, not an appendix.</strong> "Most ergonomic for
    agents" is the thesis of the whole framework; this section is where it gets
    concrete. Instructions, MCP setup, Skills, and the eval harness land here.
  </p>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)}>
    <a variant="default" href="/">← SHIP UI</a>
  </row>
</column>
