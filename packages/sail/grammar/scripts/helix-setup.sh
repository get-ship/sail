#!/usr/bin/env bash
# Overlay this checkout's queries + grammar into Helix. Re-run after generate.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
grammar=$(cd "$here/.." && pwd)
repo=$(cd "$grammar/../../.." && pwd)
helix_cfg="${HELIX_CONFIG:-$HOME/.config/helix}"
runtime="$helix_cfg/runtime"

# Helix merges ~/.config/helix/runtime over the install runtime.
# A symlink to the install would write into the install — replace it.
if [ -L "$runtime" ]; then
  echo "note: $runtime is a symlink to the Helix install; replacing with a real overlay dir"
  rm "$runtime"
fi
mkdir -p "$runtime/queries/sail" "$runtime/grammars"
ln -sfn "$grammar/queries/highlights.scm" "$runtime/queries/sail/highlights.scm"
ln -sfn "$grammar/queries/injections.scm" "$runtime/queries/sail/injections.scm"
echo "linked queries → $runtime/queries/sail"

# Absolute grammar path so `hx --grammar build` finds parser.c.
langs="$helix_cfg/languages.toml"
marker="# sail-grammar (m-sail-public helix-setup)"
block=$(
  cat <<EOF

$marker
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

if ! command -v hx >/dev/null 2>&1; then
  echo "hx not on PATH. Then: hx --grammar build"
  exit 0
fi

# Helix rebuilds every listed grammar; this machine may not have
# fetched the stock set. Success is sail.so, not a zero hx exit.
hx --grammar build >/tmp/helix-grammar-build.log 2>&1 || true
if [ -f "$runtime/grammars/sail.so" ]; then
  echo "built $runtime/grammars/sail.so"
  echo "done. Open a .sail from $repo (sail on PATH)."
  exit 0
fi
echo "sail.so missing — hx --grammar build log:"
tail -30 /tmp/helix-grammar-build.log
exit 1
