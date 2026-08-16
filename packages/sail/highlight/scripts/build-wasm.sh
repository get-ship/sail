#!/usr/bin/env bash
# Rebuild tree-sitter-sail.wasm from this repo's grammar.
# Needs a real node (not bun's node symlink) and tree-sitter-cli.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
pkg=$(cd "$here/.." && pwd)
grammar=$(cd "$pkg/../grammar" && pwd)

real_node() {
  if command -v node >/dev/null 2>&1; then
    if ! node -p 'process.versions.bun||""' 2>/dev/null | grep -q .; then
      command -v node
      return 0
    fi
  fi
  local n
  n=$(ls -1d "$HOME/.nvm/versions/node/"*/bin/node 2>/dev/null | tail -1 || true)
  if [ -n "$n" ] && [ -x "$n" ]; then
    printf '%s\n' "$n"
    return 0
  fi
  return 1
}

node_bin=$(real_node) || {
  echo "tree-sitter build --wasm needs a real node (not bun)"
  exit 1
}
PATH="$(dirname "$node_bin"):$PATH"
cli="$grammar/node_modules/tree-sitter-cli/tree-sitter"
if [ ! -x "$cli" ]; then
  echo "missing $cli — bun install in packages/sail/grammar"
  exit 1
fi
"$cli" build --wasm -o "$pkg/tree-sitter-sail.wasm" "$grammar"
echo "wrote $pkg/tree-sitter-sail.wasm"

runtime=""
for c in \
  "$pkg/node_modules/web-tree-sitter/web-tree-sitter.wasm" \
  "$pkg/../node_modules/web-tree-sitter/web-tree-sitter.wasm" \
  "$pkg/../../../node_modules/web-tree-sitter/web-tree-sitter.wasm"
do
  if [ -f "$c" ]; then
    runtime=$c
    break
  fi
done
if [ -z "$runtime" ]; then
  echo "web-tree-sitter.wasm not found — bun install in packages/sail/highlight"
  exit 1
fi
cp "$runtime" "$pkg/web-tree-sitter.wasm"
echo "wrote $pkg/web-tree-sitter.wasm"
