// Co-located component examples — the showcase contract.
//
// Each `<tag>.examples.ts` next to a component's `.sail` exports
// `examples: ComponentExample[]`. The component owns its showcase; the
// Studio components explorer, web-qa fixtures, and docs all read from
// here (one source of truth) instead of a central map that drifts.
//
// HARD RULE: fixtures are real, characterful Ship-domain content —
// agents, pipelines, worktrees, sessions, git. Never "Example",
// "Card title", "Section A", or lorem. A fixture that doesn't sell the
// component is a bug.

export interface ComponentExample {
  /** Scenario title — descriptive and specific. Never "Example N". */
  name: string
  /** Optional one-line context shown under the title. */
  description?: string
  /** Host attribute/prop values for this scenario. */
  props?: Record<string, string | boolean | number>
  /** Inner HTML — slots + content. Real, characterful markup; no placeholders. */
  slots?: string
}
