#!/usr/bin/env bash
# preflight.sh <sandbox> — fail-closed grok inspect for AGENTS.md + skill.
# Override soft path only with SAIL_LAB_PREFLIGHT_SOFT=1 (not for baseline).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

SB="${1:-}"
[[ -d "$SB" ]] || die "usage: preflight.sh <sandbox>"
require_cmd grok

ARM="$(python3 -c "import json; print(json.load(open('$SB/.lab-meta.json'))['arm'])")"
SOFT="${SAIL_LAB_PREFLIGHT_SOFT:-0}"

cd "$SB"
set +e
OUT="$(grok inspect --json 2>&1)"
INSPECT_EC=$?
set -e
printf '%s\n' "$OUT" > "$SB/.lab-preflight.json"

python3 - <<'PY' "$SB" "$ARM" "$OUT" "$SOFT" "$INSPECT_EC"
import json, sys
from pathlib import Path

sb, arm, out, soft, inspect_ec = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5])
errors = []
soft_mode = soft == "1"

# Structural file checks (always)
if arm == "sail":
    if not (Path(sb) / "AGENTS.md").exists():
        errors.append("AGENTS.md missing in sandbox")
    skill = Path(sb) / ".grok/skills/sail-authoring/SKILL.md"
    if not skill.exists() or skill.stat().st_size < 20:
        errors.append("sail-authoring skill missing or empty")

# Parse inspect JSON
data = None
try:
    data = json.loads(out)
except Exception:
    for i, c in enumerate(out):
        if c == "{":
            try:
                data = json.loads(out[i:])
                break
            except Exception:
                pass

if data is None:
    msg = "grok inspect did not return parseable JSON"
    if soft_mode:
        Path(sb, ".lab-preflight-warn.txt").write_text(msg + "\n")
    else:
        errors.append(msg)
else:
    if arm == "sail":
        # projectInstructions: fileType agents_md
        instr = data.get("projectInstructions") or data.get("project_instructions") or []
        agents_ok = False
        for item in instr if isinstance(instr, list) else []:
            if not isinstance(item, dict):
                continue
            ft = str(item.get("fileType") or item.get("file_type") or "").lower()
            path = str(item.get("path") or "")
            if ft == "agents_md" or path.rstrip("/").endswith("AGENTS.md"):
                agents_ok = True
                break
        if not agents_ok and soft_mode:
            # Blob substring fallback only when operator opts into soft preflight
            blob = json.dumps(data).lower()
            if "agents.md" in blob or '"agents_md"' in blob:
                agents_ok = True
                Path(sb, ".lab-preflight-warn.txt").write_text(
                    "agents discovery via soft blob fallback (not projectInstructions)\n"
                )
        if not agents_ok:
            errors.append("grok inspect did not discover AGENTS.md (projectInstructions)")

        skills = data.get("skills") or []
        skill_ok = False
        for s in skills if isinstance(skills, list) else []:
            if not isinstance(s, dict):
                continue
            name = str(s.get("name") or "")
            src = s.get("source") or {}
            src_type = str(src.get("type") or "") if isinstance(src, dict) else ""
            path = str(src.get("path") or "") if isinstance(src, dict) else ""
            if name == "sail-authoring" and (src_type == "project" or "sail-authoring" in path):
                skill_ok = True
                break
        if not skill_ok:
            errors.append("grok inspect did not list project skill sail-authoring")

if errors:
    print("PREFLIGHT FAIL:", "; ".join(errors), file=sys.stderr)
    if soft_mode:
        Path(sb, ".lab-preflight-warn.txt").write_text("SOFT: " + "; ".join(errors) + "\n")
        print("preflight soft-ok (SAIL_LAB_PREFLIGHT_SOFT=1)")
        sys.exit(0)
    sys.exit(1)
print("preflight ok")
PY
