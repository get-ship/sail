#!/usr/bin/env bash
# Shared paths and helpers for sail-lab harness.
set -euo pipefail

export SAIL_LAB="${SAIL_LAB:-$HOME/dev/sail-lab}"
export SAIL_REPO="${SAIL_REPO:-$HOME/dev/Ship/sail}"
export SAIL_LAB_MODEL="${SAIL_LAB_MODEL:-grok}"
export HARNESS_VER="eval2-0.1.0"

LAB="$SAIL_LAB"
REPO="$SAIL_REPO"

die() { echo "error: $*" >&2; exit 1; }

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing command: $1"
}

# Prefer python sqlite; do not require sqlite3 CLI
ensure_lab_db() {
  python3 - <<PY
import sqlite3
from pathlib import Path
lab = Path("$LAB")
schema = lab / "schema.sql"
db = lab / "lab.sqlite"
conn = sqlite3.connect(str(db))
if schema.exists():
    conn.executescript(schema.read_text())
conn.close()
PY
}

# Toolchain stamp: {git_sha}{-dirty}-r{cargo_version}
toolchain_stamp() {
  local sha dirty ver
  if git -C "$REPO" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    sha="$(git -C "$REPO" rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
    dirty=""
    if ! git -C "$REPO" diff --quiet 2>/dev/null; then dirty="-dirty"; fi
    if ! git -C "$REPO" diff --cached --quiet 2>/dev/null; then dirty="-dirty"; fi
  else
    if [[ -f "$REPO/.git" ]]; then
      sha="$(sed -n 's/.*worktrees\/\([^/]*\).*/\1/p' "$REPO/.git" 2>/dev/null || true)"
    fi
    sha="${sha:-nogit}"
    dirty="-dirty"
  fi
  ver="$(python3 - <<'PY' "$REPO"
import re, sys
from pathlib import Path
root = Path(sys.argv[1])
for p in [root/"crates/deck/deck-cli/Cargo.toml", root/"Cargo.toml"]:
    if p.exists():
        t = p.read_text()
        m = re.search(r'^version\s*=\s*"([^"]+)"', t, re.M)
        if m:
            print(m.group(1)); break
else:
    print("0.0.0")
PY
)"
  echo "${sha}${dirty}-r${ver}"
}

latest_toolchain() {
  local d
  d="$(ls -1dt "$LAB"/toolchains/*/ 2>/dev/null | head -1 || true)"
  [[ -n "$d" ]] || die "no toolchains; run harness/build_toolchain.sh"
  basename "${d%/}"
}

profile_hash() {
  local profile_path="$1"
  python3 - <<'PY' "$profile_path"
import hashlib, json, re, sys
from pathlib import Path
raw = Path(sys.argv[1]).read_text()
lines = []
for line in raw.splitlines():
    if line.strip().startswith("//"):
        continue
    lines.append(re.sub(r"\s+//.*$", "", line))
text = "\n".join(lines)
try:
    obj = json.loads(re.sub(r",\s*}", "}", re.sub(r",\s*]", "]", text)))
    text = json.dumps(obj, sort_keys=True, separators=(",", ":"))
except Exception:
    pass
print(hashlib.sha256(text.encode()).hexdigest()[:16])
PY
}

assert_repo_clean_of_lab_artifacts() {
  # Fail batch/smoke if lab run junk leaked into the sail repo tree.
  # Primary: filesystem scan (works when gitdir is broken).
  # Secondary: git status when available.
  local check="$REPO/bench/scripts/check-repo-clean.sh"
  if [[ -x "$check" ]]; then
    "$check" || die "lab artifacts leaked into sail repo (check-repo-clean.sh failed)"
  else
    # Inline scan if script not present
    if [[ -d "$REPO/bench/runs" ]] || [[ -f "$REPO/lab.sqlite" ]]; then
      die "lab artifacts under sail repo root/bench"
    fi
  fi

  if git -C "$REPO" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    local bad
    bad="$(git -C "$REPO" status --short 2>/dev/null | grep -E 'bench/runs|lab\.sqlite|\.probe-|screenshot\.png' || true)"
    if [[ -n "$bad" ]]; then
      echo "error: unexpected lab-like paths in sail repo git status:" >&2
      echo "$bad" >&2
      exit 1
    fi
  else
    echo "note: sail repo gitdir unavailable — cleanliness enforced via filesystem scan only" >&2
  fi
}
