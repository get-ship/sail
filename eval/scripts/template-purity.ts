// template-purity.ts — the SUI authoring line, made mechanical.
//
// A `.sui` template expression must be a PURE function of scope signals:
// signal reads + pure transforms (map/filter/arithmetic/ternary/member access).
// Capabilities (I/O, authority, non-determinism, non-owned mutation) and
// statements/declarations belong in `controller.ts`, reached through the ship
// SDK. A template that contains either is a FAIL — promote it to the controller.
//
// Source of truth for the line:
//   docs/ship-sui/references/docs/authoring-model.md
//   docs/ship-adrs/references/docs/sui-authoring-model.md
//
// The grader scans only TEMPLATE EXPRESSION regions — mustache `{...}`
// interpolations and `{#...}` control-flow tags — never prose text nodes or
// `<data source="ship.*()">` attributes (those SDK calls are the sanctioned
// authority path). This keeps the heuristic precise: a bare `fetch` in an
// expression fails; `ship.users.list()` in a `<data>` source does not.

interface Ban {
  // Matched against the inner text of a `{...}` expression span.
  pattern: RegExp
  label: string
}

// Capabilities, non-determinism, and in-place mutation of non-owned state.
const CAPABILITY_BANS: Ban[] = [
  { pattern: /\bfetch\s*\(/, label: 'fetch() — I/O belongs in <data>/controller' },
  { pattern: /\bXMLHttpRequest\b/, label: 'XHR — I/O belongs in the controller' },
  { pattern: /\bWebSocket\s*\(/, label: 'WebSocket — use a <data live> subscription' },
  { pattern: /\beval\s*\(/, label: 'eval — authority is controller-only' },
  { pattern: /\bFunction\s*\(/, label: 'Function() — authority is controller-only' },
  { pattern: /\bimport\s*\(/, label: 'dynamic import() — belongs in the controller' },
  { pattern: /\b(local|session)Storage\b/, label: 'web storage — read it in the controller' },
  { pattern: /\bDate\.now\s*\(/, label: 'Date.now() — non-determinism; seed in the controller' },
  { pattern: /\bMath\.random\s*\(/, label: 'Math.random() — non-determinism; seed in the controller' },
  { pattern: /\bnew\s+Date\s*\(\s*\)/, label: 'new Date() — non-determinism; seed in the controller' },
  { pattern: /\.(sort|reverse|splice|push|unshift|pop|shift)\s*\(/, label: 'in-place mutation — copy first ([...arr].sort()), in a derived' },
]

// Statements / declarations. `.sui` expressions are expressions, not blocks:
// no `const`/`let`/`var`, no `function` declaration. Promote to a `derived`.
const STATEMENT_BANS: Ban[] = [
  { pattern: /\b(const|let|var)\s+[A-Za-z_$]/, label: 'declaration in a template — promote to a controller `derived`' },
  { pattern: /\bfunction\b/, label: 'function declaration — logic belongs in the controller' },
]

const BANS: Ban[] = [...CAPABILITY_BANS, ...STATEMENT_BANS]

// Pull the inner text of every `{...}` span (mustache interpolation or control
// tag). Non-nested by design — covers the overwhelming majority of template
// expressions; deeply nested object literals are rare in templates and not
// worth a full parser in a measurement script.
function expressionSpans(suiText: string): string[] {
  const withoutComments = suiText.replace(/<!--[\s\S]*?-->/g, '')
  return (withoutComments.match(/\{[^{}]*\}/g) ?? []).map((m) => m.slice(1, -1))
}

// Return the deduplicated set of violation labels for one `.sui` file's text.
// Empty array ⇒ the template is pure (pass).
export function scanTemplatePurity(suiText: string): string[] {
  const hits = new Set<string>()
  for (const span of expressionSpans(suiText)) {
    for (const ban of BANS) {
      if (ban.pattern.test(span)) hits.add(ban.label)
    }
  }
  return [...hits]
}
