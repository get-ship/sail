#!/usr/bin/env bash
# Copy committed bench standards into ~/dev/sail-lab/ (profiles + tasks + agents + schema + fixtures).
# Also refreshes harness/payload from bench/lab-kit when present (source of truth for the instrument).
# Never copies run data into the sail repo.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BENCH="$REPO_ROOT/bench"
LAB="${SAIL_LAB:-$HOME/dev/sail-lab}"
KIT="$BENCH/lab-kit"

mkdir -p "$LAB"/{profiles,tasks,agents,runs,reports,toolchains,harness,payload,fixtures}

rsync -a --delete "$BENCH/profiles/" "$LAB/profiles/"
rsync -a --delete "$BENCH/tasks/" "$LAB/tasks/"
rsync -a --delete "$BENCH/agents/" "$LAB/agents/"
cp -f "$BENCH/schema.sql" "$LAB/schema.sql"
rsync -a "$BENCH/fixtures/" "$LAB/fixtures/"

# Instrument recoverability: lab-kit is the committed harness/payload SSOT
if [[ -d "$KIT/harness" ]]; then
  rsync -a --delete "$KIT/harness/" "$LAB/harness/"
  chmod +x "$LAB"/harness/*.{sh,py} 2>/dev/null || true
fi
if [[ -d "$KIT/payload" ]]; then
  rsync -a "$KIT/payload/" "$LAB/payload/"
  if [[ -f "$LAB/payload/sail/.grok/hooks/log.sh" ]]; then
    chmod +x "$LAB/payload/sail/.grok/hooks/log.sh"
  fi
fi

# Seed sqlite if missing (python — sqlite3 CLI may be absent)
if [[ ! -f "$LAB/lab.sqlite" ]]; then
  python3 - <<PY
import sqlite3
from pathlib import Path
lab = Path("$LAB")
conn = sqlite3.connect(str(lab / "lab.sqlite"))
conn.executescript((lab / "schema.sql").read_text())
conn.close()
print("created", lab / "lab.sqlite")
PY
fi

echo "synced bench → $LAB (profiles, tasks, agents, schema, fixtures$([[ -d $KIT/harness ]] && echo ', lab-kit harness/payload'))"
