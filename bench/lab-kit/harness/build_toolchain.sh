#!/usr/bin/env bash
# Build release deck-cli + sail-mcp + sail-compiler; strip into toolchains/<stamp>/bin/.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=lib.sh
source "$SCRIPT_DIR/lib.sh"

require_cmd cargo
require_cmd strip

STAMP="$(toolchain_stamp)"
DEST="$LAB/toolchains/$STAMP/bin"
mkdir -p "$DEST"

echo "== building release deck-cli + sail-mcp + sail-compiler (stamp=$STAMP) =="
(
  cd "$REPO"
  cargo build --release -p deck-cli -p sail-mcp -p sail-compiler
)

SAIL_SRC="$REPO/target/release/sail"
MCP_SRC="$REPO/target/release/sail-mcp"
COMPILER_SRC="$REPO/target/release/sail-compiler"
[[ -x "$SAIL_SRC" ]] || die "missing $SAIL_SRC"
[[ -x "$MCP_SRC" ]] || die "missing $MCP_SRC"
[[ -x "$COMPILER_SRC" ]] || die "missing $COMPILER_SRC (required for P0 generated skill)"

cp -f "$SAIL_SRC" "$DEST/sail"
cp -f "$MCP_SRC" "$DEST/sail-mcp"
cp -f "$COMPILER_SRC" "$DEST/sail-compiler"
strip "$DEST/sail" "$DEST/sail-mcp" "$DEST/sail-compiler"
chmod +x "$DEST/sail" "$DEST/sail-mcp" "$DEST/sail-compiler"

cat > "$LAB/toolchains/$STAMP/META.txt" <<EOF
stamp=$STAMP
built_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
repo=$REPO
git_head=$(git -C "$REPO" rev-parse HEAD 2>/dev/null || echo unknown)
bins=sail,sail-mcp,sail-compiler
EOF

echo "toolchain ready: $DEST"
echo "$STAMP"
