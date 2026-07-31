#!/usr/bin/env bash
# Assert sail repo has no lab run artifacts under bench/ or repo root.
# Invoked by lab harness after batch/smoke. Fails (exit 1) on leakage.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
bad=0

scan() {
  local root="$1"
  while IFS= read -r -d '' f; do
    echo "leaked run artifact: $f" >&2
    bad=1
  done < <(find "$root" \( \
    -name 'screenshot.png' -o -name 'lab.sqlite' -o -name '.probe-events.jsonl' \
    -o -name '.probe-shot.json' -o -name '.lab-harvest.json' \
  \) -print0 2>/dev/null)
}

scan "$REPO_ROOT/bench"

# Repo-root leakage (never write lab data into sail tree)
for p in \
  "$REPO_ROOT/lab.sqlite" \
  "$REPO_ROOT/runs" \
  "$REPO_ROOT/bench/runs" \
  "$REPO_ROOT/toolchains"
do
  if [[ -e "$p" ]]; then
    echo "leaked path: $p" >&2
    bad=1
  fi
done

# Any run-like dirs under bench
if find "$REPO_ROOT/bench" -type d -name 'runs' 2>/dev/null | grep -q .; then
  echo "leaked bench/**/runs directory" >&2
  bad=1
fi

if [[ $bad -ne 0 ]]; then
  exit 1
fi
echo "bench standards tree has no run artifacts"
