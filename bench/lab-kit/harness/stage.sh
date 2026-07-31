#!/usr/bin/env bash
# stage.sh — materialize a fresh sandbox for one run.
# Usage: stage.sh <task_id> <arm> <n> [toolchain_stamp]
# Prints the sandbox path on stdout (last line).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

TASK_ID="${1:-}"
ARM="${2:-}"
N="${3:-1}"
STAMP="${4:-}"
[[ -n "$TASK_ID" && -n "$ARM" ]] || die "usage: stage.sh <task_id> <arm> <n> [stamp]"

TASK_DIR="$LAB/tasks/$TASK_ID"
[[ -d "$TASK_DIR" ]] || die "unknown task: $TASK_ID (sync bench first)"

if [[ -z "$STAMP" ]]; then
  STAMP="$(latest_toolchain)"
fi
TC_BIN="$LAB/toolchains/$STAMP/bin"
[[ -x "$TC_BIN/sail" ]] || die "missing toolchain bin: $TC_BIN/sail"

TS="$(date +%Y%m%d-%H%M%S)"
SB="$LAB/runs/${TS}-${TASK_ID}-${ARM}-${N}"
mkdir -p "$SB"

TASK_JSON="$TASK_DIR/task.jsonc"
SCAFFOLD="$(python3 - <<'PY' "$TASK_JSON"
import json, re, sys
from pathlib import Path
raw = Path(sys.argv[1]).read_text()
lines = [ln for ln in raw.splitlines() if not ln.strip().startswith("//")]
text = "\n".join(lines)
text = re.sub(r",\s*}", "}", re.sub(r",\s*]", "]", text))
print(json.loads(text).get("scaffold", "blank"))
PY
)"

PROFILE_ID="p0"
PROFILE_PATH="$LAB/profiles/${PROFILE_ID}.jsonc"
[[ -f "$PROFILE_PATH" ]] || die "missing profile $PROFILE_PATH"
PHASH="$(profile_hash "$PROFILE_PATH")"

# skill_source recorded into meta for harvest flags
SKILL_SOURCE=""
MCP_FALSE=1

# Portable deny roots ($HOME / $SAIL_REPO — not hard-coded usernames)
DENY_SSH="$HOME/.ssh"
DENY_GPG="$HOME/.gnupg"
# Ship tree: prefer $HOME/dev/Ship if present, else the configured repo root
if [[ -d "$HOME/dev/Ship" ]]; then
  DENY_SHIP="$HOME/dev/Ship"
else
  DENY_SHIP="$REPO"
fi

write_sandbox_toml() {
  local out="$1"
  # Collect unique deny paths. Do NOT list a path and its ancestor (bwrap fails
  # with "Can't mkdir parents … Read-only file system" on nested denies).
  local -a denies=("$DENY_SSH" "$DENY_GPG")
  denies+=("$DENY_SHIP")
  # Only add REPO if it is not under DENY_SHIP
  case "$REPO" in
    "$DENY_SHIP"|"$DENY_SHIP"/*) ;;
    *) denies+=("$REPO") ;;
  esac
  {
    echo '[profiles.sail-eval-iso]'
    echo 'extends = "workspace"'
    printf 'deny = ["**/.env", "**/*.pem"'
    local p
    for p in "${denies[@]}"; do
      [[ -n "$p" ]] || continue
      printf ', "%s"' "$p"
    done
    for d in "$LAB"/runs/*/; do
      [[ "$d" == "$SB/" ]] && continue
      [[ -d "$d" ]] || continue
      printf ', "%s"' "${d%/}"
    done
    if [[ -d "$HOME/dev/sail-probe/runs" ]]; then
      for d in "$HOME/dev/sail-probe"/runs/*/; do
        [[ -d "$d" ]] || continue
        printf ', "%s"' "${d%/}"
      done
    fi
    echo ']'
  } > "$out"
}

# Post-process generated skill: rewrite MCP tool table + frontmatter to CLI when mcp:false
rewrite_skill_cli() {
  local skill="$1"
  [[ -f "$skill" ]] || return 0
  python3 - <<'PY' "$skill"
import re, sys
from pathlib import Path
p = Path(sys.argv[1])
text = p.read_text()
# Frontmatter description often steers agents harder than the body — rewrite MCP hints
def rewrite_frontmatter(src: str) -> str:
    m = re.match(r"^---\n(.*?)\n---\n?", src, flags=re.S)
    if not m:
        return src
    fm = m.group(1)
    # description: line (may be long single line)
    def desc_sub(mm: re.Match) -> str:
        d = mm.group(1)
        d = re.sub(
            r"use MCP\s+sail_surface\s*/\s*sail_check for full tables",
            "use `./.sail-bin/sail op SailAuthoring/Surface` / `./.sail-bin/sail check` for full tables",
            d,
            flags=re.I,
        )
        d = re.sub(r"\bMCP\s+sail_\w+\b", "`./.sail-bin/sail` CLI", d, flags=re.I)
        d = re.sub(r"\bMCP\b", "CLI", d)
        d = re.sub(r"\s{2,}", " ", d).strip()
        return f"description: {d}"
    fm2 = re.sub(r"^description:\s*(.+)$", desc_sub, fm, count=1, flags=re.M)
    return "---\n" + fm2 + "\n---\n" + src[m.end():]

text = rewrite_frontmatter(text)
# Replace progressive-disclosure MCP tool table rows with CLI equivalents
cli_table = """| need | tool |
| --- | --- |
| focused surface tables | `./.sail-bin/sail op SailAuthoring/Surface --json '{"topic":"…"}'` (topics: core, layout, style, tokens, l1, control-flow) |
| lint/compile diagnostics | `./.sail-bin/sail check <path>` |
| structured grammar-aware edit | prefer edit + check; MCP sail_edit unavailable in this lab profile |
| regenerate this skill | host-side only — do not regenerate mid-run |
"""
text2 = re.sub(
    r"\| need \| tool \|.*?\| regenerate this skill \|.*?\|",
    cli_table.strip(),
    text,
    count=1,
    flags=re.S,
)
banner = (
    "\n> **Lab profile (mcp: false):** project MCP is unavailable. "
    "Use `./.sail-bin/sail` CLI only — do not wait for MCP tools.\n\n"
)
if "Lab profile (mcp: false)" not in text2:
    text2 = re.sub(r"(# [^\n]+\n)", r"\1" + banner, text2, count=1)
text2 = re.sub(
    r"MCP server:.*?(\n\n|\n(?=#))",
    "MCP server: **disabled for this lab profile.** Use `./.sail-bin/sail`.\n\n",
    text2,
    count=1,
    flags=re.S,
)
# Remaining body "MCP `sail_…`" tool mentions → CLI
text2 = re.sub(
    r"MCP\s+`sail_(surface|check|edit)`",
    r"`./.sail-bin/sail` (\\1 via op/check)",
    text2,
)
p.write_text(text2)
PY
}

case "$ARM" in
  sail)
    cp -a "$LAB/payload/sail/." "$SB/"
    AGENTS_REL="$(python3 - <<'PY' "$PROFILE_PATH"
import json, re, sys
from pathlib import Path
raw = Path(sys.argv[1]).read_text()
lines = [ln for ln in raw.splitlines() if not ln.strip().startswith("//")]
text = "\n".join(lines)
text = re.sub(r",\s*}", "}", re.sub(r",\s*]", "]", text))
print(json.loads(text).get("agents_md", "agents/agents-v2.md"))
PY
)"
    if [[ -f "$LAB/$AGENTS_REL" ]]; then
      cp -f "$LAB/$AGENTS_REL" "$SB/AGENTS.md"
    elif [[ -f "$REPO/bench/$AGENTS_REL" ]]; then
      cp -f "$REPO/bench/$AGENTS_REL" "$SB/AGENTS.md"
    else
      die "agents_md not found: $AGENTS_REL"
    fi

    # Banner: MCP unavailable (P0 mcp:false)
    if ! grep -q 'MCP is unavailable' "$SB/AGENTS.md" 2>/dev/null; then
      {
        echo ""
        echo "## Lab profile note"
        echo ""
        echo "**MCP is unavailable** in this sandbox (\`mcp: false\`). Do not use or wait for"
        echo "MCP tools. Use \`./.sail-bin/sail check\` and \`./.sail-bin/sail op\` only."
        echo ""
      } >> "$SB/AGENTS.md"
    fi

    if [[ "$SCAFFOLD" == "overlay" && -d "$TASK_DIR/overlay" ]]; then
      cp -a "$TASK_DIR/overlay/." "$SB/"
    fi

    if [[ -d "$TASK_DIR/reference" ]]; then
      mkdir -p "$SB/reference"
      cp -a "$TASK_DIR/reference/." "$SB/reference/"
    fi

    mkdir -p "$SB/.sail-bin"
    cp -al "$TC_BIN/sail" "$SB/.sail-bin/sail"
    if [[ -x "$TC_BIN/sail-mcp" ]]; then
      cp -al "$TC_BIN/sail-mcp" "$SB/.sail-bin/sail-mcp"
    fi

    # P0: skill_source=generated — fail closed without sail-compiler
    mkdir -p "$SB/.grok/skills/sail-authoring"
    SKILL_OUT="$SB/.grok/skills/sail-authoring/SKILL.md"
    GENERATED=0
    if [[ -x "$TC_BIN/sail-compiler" ]]; then
      if "$TC_BIN/sail-compiler" skill --dir "$SB" --out "$SKILL_OUT"; then
        if [[ -s "$SKILL_OUT" ]] && grep -q 'sail-authoring\|Authoring SAIL\|Authoring sail' "$SKILL_OUT"; then
          GENERATED=1
          SKILL_SOURCE="generated:${STAMP}"
        fi
      fi
    fi

    if [[ "$GENERATED" -ne 1 ]]; then
      # Fail closed for P0: do not stage a silent stub as if generated
      if [[ "${SAIL_LAB_ALLOW_SKILL_FALLBACK:-0}" == "1" ]]; then
        if [[ -f "$HOME/dev/sail-probe/payload/.grok/skills/sail-authoring/SKILL.md" ]]; then
          cp -f "$HOME/dev/sail-probe/payload/.grok/skills/sail-authoring/SKILL.md" "$SKILL_OUT"
          SKILL_SOURCE="probe-fallback"
        else
          cat > "$SKILL_OUT" <<'EOF'
---
name: sail-authoring
description: Author .sail in THIS directory. Generated skill unavailable — use CLI surface tools.
---
# Authoring SAIL here
Use `./.sail-bin/sail check` and `./.sail-bin/sail op SailAuthoring/Surface`.
No CSS. L1 first. Fix every diagnostic.
EOF
          SKILL_SOURCE="stub"
        fi
        echo "WARN: skill not generated; using $SKILL_SOURCE (SAIL_LAB_ALLOW_SKILL_FALLBACK=1)" >&2
      else
        rm -rf "$SB"
        die "P0 requires generated skill; missing/failed sail-compiler skill gen (toolchain $STAMP). Build with harness/build_toolchain.sh (includes -p sail-compiler). Override only with SAIL_LAB_ALLOW_SKILL_FALLBACK=1."
      fi
    fi

    # mcp:false — rewrite skill tooling table to CLI
    rewrite_skill_cli "$SKILL_OUT"

    rm -f "$SB/.mcp.json"
    chmod +x "$SB/.grok/hooks/log.sh"
    write_sandbox_toml "$SB/.grok/sandbox.toml"
    ;;

  react-tw)
    BASE="$LAB/payload/react-base"
    [[ -f "$BASE/package.json" ]] || die "react-base missing; run harness/prepare_react_base.sh"
    rsync -a --exclude node_modules "$BASE/" "$SB/"
    if [[ -d "$BASE/node_modules" ]]; then
      cp -al "$BASE/node_modules" "$SB/node_modules"
    else
      echo "WARN: react-base has no node_modules; run prepare_react_base.sh" >&2
    fi
    if [[ -d "$TASK_DIR/reference" ]]; then
      mkdir -p "$SB/reference"
      cp -a "$TASK_DIR/reference/." "$SB/reference/"
    fi
    mkdir -p "$SB/.grok/hooks"
    cp -f "$LAB/payload/sail/.grok/hooks/log.sh" "$SB/.grok/hooks/log.sh"
    cp -f "$LAB/payload/sail/.grok/hooks/eval-logger.json" "$SB/.grok/hooks/eval-logger.json"
    chmod +x "$SB/.grok/hooks/log.sh"
    write_sandbox_toml "$SB/.grok/sandbox.toml"
    SKILL_SOURCE="n/a"
    ;;

  *)
    die "unknown arm: $ARM (sail|react-tw)"
    ;;
esac

PROMPT="$(python3 - <<'PY' "$TASK_JSON"
import json, re, sys
from pathlib import Path
raw = Path(sys.argv[1]).read_text()
lines = [ln for ln in raw.splitlines() if not ln.strip().startswith("//")]
text = "\n".join(lines)
text = re.sub(r",\s*}", "}", re.sub(r",\s*]", "]", text))
print(json.loads(text)["prompt"])
PY
)"
printf '%s\n' "$PROMPT" > "$SB/.lab-prompt.txt"

python3 - <<PY
import json
from pathlib import Path
meta = {
  "task_id": "$TASK_ID",
  "arm": "$ARM",
  "n": int("$N"),
  "toolchain": "$STAMP",
  "profile_id": "$PROFILE_ID",
  "profile_hash": "$PHASH",
  "harness_ver": "$HARNESS_VER",
  "model": "$SAIL_LAB_MODEL",
  "scaffold": "$SCAFFOLD",
  "skill_source": "$SKILL_SOURCE",
  "mcp": False,
}
Path("$SB/.lab-meta.json").write_text(json.dumps(meta, indent=2) + "\n")
PY

git init -q "$SB"
git -C "$SB" add -A
git -C "$SB" -c user.email=lab@sail -c user.name=lab commit -qm baseline

echo "$SB"
