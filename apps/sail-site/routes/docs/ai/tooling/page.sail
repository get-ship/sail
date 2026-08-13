<!--
  AI tooling — sail-lint, sail-compiler, sail-lsp (one Rust grammar, several surfaces).
-->

<scope>
  style page = { box: size({ max-w: breakpoint-tablet }).space({ mx: auto }) }
  style eyebrow = { box: space({ mb: 2 }), paint: ink({ color: accent }), text: type({ family: mono, size: xs, tracking: wide, case: uppercase }) }
  style lead = { paint: ink({ color: muted-foreground }), text: type({ size: lg, leading: relaxed }) }
  style footNav = { box: space({ pad-t: 4, mt: 8 }), paint: border({ color: border, width: 1, edge: t }) }
</scope>

<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { lintCmd, lintOutput, compileCmd, lspJson } from '../../../../src/compositions/ai-tooling-snippets'

export default controller((_ctx: ControllerScope<Scope>) => ({ lintCmd, lintOutput, compileCmd, lspJson }))
</controller>

<article style(page)>
  <p style(eyebrow)>AI</p>
  <h1>The LSP — diagnostics on tap</h1>
  <p style(lead)>
    Sail’s language intelligence is the Rust compiler —
    <code>sail_compiler::analyze</code> (diagnostics + scope symbols + dependencies)
    and <code>check_all</code> (every diagnostic, not just the first). It’s the
    same grammar the build uses, exposed to agents through two binary commands.
  </p>

  <h2>Fast per-edit lint</h2>
  <code-block code={lintCmd} lang="bash"></code-block>
  <p>
    Exits 1 on findings. Covers grammar errors, missing bindings, unstable-key
    warnings, purity violations, CSS token errors, and prop-value mismatches.
    Run this in your edit loop — it’s the cheapest check and catches the most
    common authoring errors.
  </p>

  <p>Output on a file with issues:</p>
  <code-block code={lintOutput} lang="text"></code-block>
  <p>
    <code>severity</code> is <code>error</code> (compile-blocking) or
    <code>warning</code> (compiles but suspicious). The rule id is in brackets
    — useful for filtering or mapping to your editor’s diagnostic provider.
    Positions are 1-based.
  </p>

  <h2>Full compile</h2>
  <code-block code={compileCmd} lang="bash"></code-block>
  <p>
    Runs the full pipeline (parse → check → lint → lower → emit) and aborts on
    errors. More thorough than the lint check — catches lowering-level issues
    the fast pass misses. Also emits the compiled JS, <code>.sail.d.ts</code>,
    and SSR HTML into the project mirror tree. Use this after the lint check
    is clean.
  </p>

  <h2>The <code>sail-lsp</code> binary (editors)</h2>
  <p>
    For editors — VS Code, Neovim, Helix, or any LSP-capable client — the
    <code>sail-lsp</code> binary provides the same intelligence over stdio.
    Same Rust grammar; hover, completion, and go-to-definition on top. Register
    it for <code>.sail</code> files with a <code>.lsp.json</code> at your
    project root and any LSP-capable editor attaches automatically:
  </p>
  <code-block code={lspJson} lang="json"></code-block>

  <h2>Why this matters</h2>
  <p>
    There is <strong>one</strong> source of truth — the Rust
    <code>sail_compiler</code> grammar — flowing through multiple surfaces:
  </p>
  <ul>
    <li><strong><code>sail-lint</code></strong> — fast per-edit binary for agents and CI</li>
    <li><strong><code>sail-compiler</code></strong> — full compile with emit, used by the deck bundler</li>
    <li><strong><code>sail-lsp</code></strong> — LSP server for editors over stdio</li>
    <li><strong>WASM compiler</strong> (<code>@ship/sail-compiler-wasm</code>) — in-browser diagnostics for the docs and any embedded editor</li>
  </ul>
  <p>
    A human typing in the docs site, an agent running <code>sail-lint</code>,
    and an editor on your laptop all see the <em>same</em> diagnostics —
    because they’re all the same compiler. No second grammar to drift out of
    sync.
  </p>

  <row style(footNav)>
    <a href="/docs/ai/authoring">← Authoring</a>
    <a href="/docs/ai/eval">Eval →</a>
  </row>
</article>
