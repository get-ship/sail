#!/usr/bin/env bash
# run_one.sh <task_id> <arm> <n> [toolchain_stamp]
# Stage → preflight → headless grok → post-run harvest.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

TASK_ID="${1:-}"
ARM="${2:-}"
N="${3:-1}"
STAMP="${4:-}"
[[ -n "$TASK_ID" && -n "$ARM" ]] || die "usage: run_one.sh <task_id> <arm> <n> [stamp]"

require_cmd grok
require_cmd python3
ensure_lab_db

# Ensure standards present
[[ -d "$LAB/tasks/$TASK_ID" ]] || "$SCRIPT_DIR/sync.sh"

if [[ -z "$STAMP" ]]; then
  if [[ ! -d "$LAB/toolchains" ]] || [[ -z "$(ls -A "$LAB/toolchains" 2>/dev/null || true)" ]]; then
    "$SCRIPT_DIR/build_toolchain.sh"
  fi
  STAMP="$(latest_toolchain)"
fi

echo "== stage $TASK_ID / $ARM / n=$N (toolchain=$STAMP) =="
SB="$("$SCRIPT_DIR/stage.sh" "$TASK_ID" "$ARM" "$N" "$STAMP" | tail -1)"
echo "sandbox: $SB"

echo "== preflight =="
"$SCRIPT_DIR/preflight.sh" "$SB"

PROMPT="$(cat "$SB/.lab-prompt.txt")"
START_MS="$(python3 -c 'import time; print(int(time.time()*1000))')"
START_ISO="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Record start in meta
python3 - <<PY
import json
from pathlib import Path
p = Path("$SB/.lab-meta.json")
m = json.loads(p.read_text())
m["started_at"] = "$START_ISO"
m["run_dir"] = "$SB"
p.write_text(json.dumps(m, indent=2) + "\n")
PY

echo "== grok headless =="
# Headless: -p / --single; unattended permissions; sandbox profile from project
set +e
(
  cd "$SB"
  grok -p "$PROMPT" \
    --trust \
    --sandbox sail-eval-iso \
    --disable-web-search \
    --no-memory \
    --permission-mode bypassPermissions \
    --cwd "$SB" \
    > "$SB/.lab-stdout.txt" 2> "$SB/.lab-stderr.txt"
)
GROK_EC=$?
set -e

END_MS="$(python3 -c 'import time; print(int(time.time()*1000))')"
WALL_MS=$((END_MS - START_MS))
echo "grok exit=$GROK_EC wall_ms=$WALL_MS"

python3 - <<PY
import json
from pathlib import Path
p = Path("$SB/.lab-meta.json")
m = json.loads(p.read_text())
m["finished_at"] = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
m["wall_ms"] = $WALL_MS
m["grok_exit"] = $GROK_EC
p.write_text(json.dumps(m, indent=2) + "\n")
PY

echo "== harvest =="
python3 "$SCRIPT_DIR/harvest.py" "$SB"

assert_repo_clean_of_lab_artifacts
echo "done: $SB"
echo "$SB"
