#!/usr/bin/env bash
# CONTRIBUTOR / this-machine overlay. Not product. Not OSS install.
# Writes ~/.config/helix and compiles sail.so for dogfooding this checkout.
#
# Strangers use: sail on PATH + the languages.toml snippet in README.md.
#
# Never replace ~/.config/helix/runtime. If that path is a symlink to the
# Helix install (common), write into it. Creating a new empty runtime dir
# hides stock themes/grammars.
#
# Never run bare `hx --grammar build` — Helix then rebuilds *every* language
# and, without `grammars/sources/*`, deletes the stock .so files. Controller
# TypeScript injection needs `typescript.so`. We compile sail.so ourselves.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
grammar=$(cd "$here/.." && pwd)
repo=$(cd "$grammar/../../.." && pwd)
helix_cfg="${HELIX_CONFIG:-$HOME/.config/helix}"
runtime="$helix_cfg/runtime"

# tree-sitter-cli evaluates grammar.js with node. bun's `node` symlink
# cannot load it (`Failed to load grammar.js`). Prefer a real Node.
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

if [ ! -f "$grammar/src/parser.c" ] || [ "$grammar/grammar.js" -nt "$grammar/src/parser.c" ]; then
  node_bin=$(real_node) || {
    echo "tree-sitter generate needs a real node (not bun's node symlink)"
    exit 1
  }
  echo "grammar.js newer than parser.c — generating with $node_bin"
  (
    cd "$grammar"
    PATH="$(dirname "$node_bin"):$PATH" \
      "$grammar/node_modules/tree-sitter-cli/tree-sitter" generate
  )
fi

if [ -L "$runtime" ]; then
  runtime=$(readlink -f "$runtime")
  echo "using Helix runtime symlink target $runtime"
elif [ ! -d "$runtime" ]; then
  # Do not mkdir a blank overlay — Helix would treat it as the whole runtime.
  if [ -d "${HELIX_RUNTIME:-}" ]; then
    runtime="$HELIX_RUNTIME"
  elif [ -d "$(dirname "$(command -v hx)")/../lib/helix" ]; then
    runtime=$(cd "$(dirname "$(command -v hx)")/../lib/helix" && pwd)
  else
    echo "no Helix runtime found (no $helix_cfg/runtime, no HELIX_RUNTIME)"
    exit 1
  fi
  echo "using Helix install runtime $runtime"
fi

mkdir -p "$runtime/queries/sail" "$runtime/grammars"
ln -sfn "$grammar/queries/highlights.scm" "$runtime/queries/sail/highlights.scm"
ln -sfn "$grammar/queries/injections.scm" "$runtime/queries/sail/injections.scm"
echo "linked queries → $runtime/queries/sail"

sail_bin="sail"
if [ -x "$repo/target/debug/sail" ]; then
  sail_bin="$repo/target/debug/sail"
elif command -v sail >/dev/null 2>&1; then
  sail_bin=$(command -v sail)
fi

langs="$helix_cfg/languages.toml"
marker="# sail-grammar (m-sail-public helix-setup)"
block=$(
  cat <<EOF

$marker
[language-server.sail]
command = "$sail_bin"
args = ["lsp"]

[[language]]
name = "sail"
scope = "source.sail"
file-types = ["sail"]
comment-token = "<!--"
indent = { tab-width = 2, unit = "  " }
language-servers = ["sail"]
grammar = "sail"

[[grammar]]
name = "sail"
source = { path = "$grammar" }
EOF
)
if [ -f "$langs" ] && grep -qF "$marker" "$langs"; then
  echo "sail grammar already in $langs"
elif [ -f "$langs" ]; then
  printf '%s\n' "$block" >>"$langs"
  echo "appended sail grammar to $langs"
else
  printf '%s\n' "$block" >"$langs"
  echo "wrote $langs"
fi

# Compile sail.so in place. Do not call `hx --grammar build` (all languages).
if ! command -v cc >/dev/null 2>&1; then
  echo "cc not on PATH — cannot compile $runtime/grammars/sail.so"
  exit 1
fi
cc -shared -fPIC -O2 -I "$grammar/src" -o "$runtime/grammars/sail.so" "$grammar/src/parser.c"
echo "built $runtime/grammars/sail.so"

# <controller> injects TypeScript. Helix 25's `hx --grammar fetch|build`
# ignores the language argument and rebuilds *every* grammar — do not call it.
if [ ! -f "$runtime/grammars/typescript.so" ]; then
  echo "warn: $runtime/grammars/typescript.so missing — controller body will not highlight"
fi

echo "done. Open a .sail from $repo, then :config-reload (or restart hx)."
echo "Theme is Helix's, not this script. Do not run \`hx --grammar build\`."
