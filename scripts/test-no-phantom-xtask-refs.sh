#!/usr/bin/env bash
# Regression test: the repo must not reference the non-existent `xtask` crate
# (only `xtask-deck` exists) nor the phantom `gen-*` task family
# (gen-icons is the only one in xtask-deck; gen-tokens/gen-types/gen-skills/
# gen-fixtures/gen-ops/gen-sdk have never landed).
#
# The xtask carve was incomplete; this guard keeps the docs honest. New
# occurrences are a doc-drift bug — fix the reference (the real command is
# `cargo run -p xtask-deck -- gen-icons` or `cargo run -p deck-cli -- build-assets`).
#
# Exit 0 on clean, 1 on any phantom match.
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

# Files we must NOT scan:
#   - this script (it intentionally lists the phantom names as data)
#   - harness / skill / user content under `.claude/` (the dispatch.sh comment
#     explicitly acknowledges the multi-repo split — that script lives in
#     the outer `ship` repo and the `xtask` ref is intentional residue)
#   - any node_modules / target / .deck / .artifacts / generated dirs
#   - any committed fixture (tokens.json header is updated separately)
excludes=(
  --exclude-dir=node_modules
  --exclude-dir=target
  --exclude-dir=.deck
  --exclude-dir=.artifacts
  --exclude-dir=.git
  --exclude-dir=.claude
  --exclude=test-no-phantom-xtask-refs.sh
  --exclude=tokens.json
  --exclude=generated.ts
  --exclude=*.d.ts
  --exclude=*.sui.d.ts
)

fail=0

# Pattern 1: bare `cargo xtask` (no `-deck` suffix) or `cargo run -p xtask `
# (note the trailing space — `-p xtask` would also match `-p xtask-deck`).
if grep -rIn "${excludes[@]}" \
    -E '(^|[^A-Za-z0-9_-])(cargo xtask(\b|$)|cargo run -p xtask )' \
    -- . 2>/dev/null; then
  echo "✗ phantom xtask crate reference (must be xtask-deck)" >&2
  fail=1
fi

# Pattern 2: phantom gen-* tasks. Allow `gen-icons` (the only one that exists);
# flag any other gen-<name> in a cargo xtask / xtask-deck context.
if grep -rIn "${excludes[@]}" \
    -E 'xtask(-deck)?\b[^]]*\bgen-(tokens|types|skills|fixtures|ops|sdk)\b' \
    -- . 2>/dev/null; then
  echo "✗ phantom gen-* task (only gen-icons exists in xtask-deck)" >&2
  fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "✓ no phantom xtask crate / gen-* task references"
  exit 0
fi
exit 1
