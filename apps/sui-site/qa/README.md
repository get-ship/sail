# SUI QA tooling

Deterministic, agent-native component QA. Detection is programmatic (no
vision); agents fix what it flags.

## Tools (CLI now; MCP wrapper later)
- `node qa/probe.mjs <tag> [--theme=a,b]` — layout probe for ONE component:
  structured box-model + defect flags (OUTSIDE-STAGE, OVERFLOW-X,
  LOW-CONTRAST via WCAG). Scoped to the specimen subtree — no page/docs noise.
- `node qa/probe.mjs --sweep [--theme=…]` — all components, CLUSTERED by
  defect type → answers "few shared bugs vs many unique".
- `qa/sweep.mjs` — earlier broad sweep (superseded by probe --sweep).
- MCP: chrome-devtools-mcp added to /.mcp.json (CDP access for agents).

## Finding so far
Default (closed/unfocused) states are CLEAN across 80 components × 3 themes
(geometry + contrast). So defects are NOT unique-per-component — they cluster
in a FEW categories: (1) gallery preview strategy (overlays render closed/
empty, disclosure dumps content), (2) INTERACTIVE states (open color-picker
palette off-screen), (3) possible theme×state combos. The fix is bounded.

## The one missing capability: STATE-DRIVING
The probe must drive each component into its states before asserting —
from the CEM manifest (tone/size/disabled/open as attributes) + interaction
(click-to-open, focus). That's what catches the open-overlay / focused-input
bugs. Next build.

## Severity model (borrowed from gstack qa taxonomy)
critical (blocks/crashes) > high (feature unusable) > medium (works, visible
problem) > cosmetic. Categories: Visual/UI, Functional, UX, Content, Perf,
Console/Errors, Accessibility (contrast). Check EVERY state: empty, loading,
error, full/overflow, open, focused, disabled.
