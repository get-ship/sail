#!/usr/bin/env bash
# Materialize ~/dev/sail-lab harness + payload from committed bench/lab-kit/.
# Standards (profiles/tasks/agents/schema/fixtures) come from sync-to-lab.sh.
#
# Source of truth for the instrument scripts:
#   $SAIL_REPO/bench/lab-kit/{harness,payload}/
# Live lab (data + stamped toolchains + runs):
#   $SAIL_LAB (default ~/dev/sail-lab)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BENCH="$REPO_ROOT/bench"
KIT="$BENCH/lab-kit"
LAB="${SAIL_LAB:-$HOME/dev/sail-lab}"

if [[ ! -d "$KIT/harness" ]]; then
  echo "error: missing $KIT/harness — lab kit not vendored under bench/" >&2
  exit 1
fi

mkdir -p "$LAB"/{harness,payload,profiles,tasks,agents,runs,reports,toolchains,fixtures}

rsync -a --delete "$KIT/harness/" "$LAB/harness/"
# payload: do not wipe react-base node_modules if already installed
rsync -a "$KIT/payload/" "$LAB/payload/"
chmod +x "$LAB"/harness/*.{sh,py} 2>/dev/null || true
if [[ -f "$LAB/payload/sail/.grok/hooks/log.sh" ]]; then
  chmod +x "$LAB/payload/sail/.grok/hooks/log.sh"
fi

# Sync standards after kit
SAIL_LAB="$LAB" "$BENCH/scripts/sync-to-lab.sh"

echo "bootstrapped lab kit → $LAB"
echo "next: cd $LAB && ./harness/build_toolchain.sh && ./harness/prepare_react_base.sh"
