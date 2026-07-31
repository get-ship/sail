# Sail authoring

You are authoring a **sail** UI page. Sail is a UI language with a strict
compiler — it is NOT HTML, JSX, or markdown, and it does not use CSS.

## Required workflow

1. **Read the skill FIRST**: `.grok/skills/sail-authoring/SKILL.md` — the
   core rules and hard constraints. Do this before writing any code.
2. Write the page(s) under `routes/` as specified by the task prompt.
3. **Verify**: run `./.sail-bin/sail check <path>` for every `.sail` file you
   touch. Fix EVERY diagnostic — errors and warnings.
4. Repeat 2–3 until the check is fully clean.

## Hard rules

- Off-list values are compile errors that name the allowed list. Trust the
  compiler over your memory. When unsure what exists, run
  `./.sail-bin/sail op SailAuthoring/Surface --json '{"topic":"l1"}'`
  (topics: core, layout, style, tokens, l1, control-flow) — do NOT guess.
- No CSS, no `<style>` blocks, no inline styles. The vocabulary is the
  styling surface.
- **NEVER declare done while `sail check` reports anything.** "It compiles
  with warnings" is not done. A layout that clips, overflows, or collapses
  is broken even if it renders.
- Run the check after EVERY edit, not once at the end.

## Vocabulary discipline

- **L1 first.** Prefer L1 elements over raw HTML tags for anything they
  cover: `<card>` for sections, `<field>` for labeled form controls,
  `<button>` for actions. Raw HTML tags (`div`, `span`, `label`, raw `h1`)
  are a last resort for gaps — not the default. Inventory: surface topic
  `l1`.

## Lab profile note

**MCP is unavailable** in this sandbox (`mcp: false`). Do not use or wait
for MCP tools (`sail_surface`, `sail_check`, …). Use `./.sail-bin/sail check`
and `./.sail-bin/sail op SailAuthoring/Surface` only.
